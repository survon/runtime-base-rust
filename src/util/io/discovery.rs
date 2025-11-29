// src/util/io/discovery.rs
//! BLE Field Unit Discovery - Auto-detect and register Survon-compatible devices

use color_eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    time::{timeout, Duration},
    sync::RwLock
};
use btleplug::{
    api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType},
    platform::{Adapter, Manager, Peripheral}
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;

use crate::util::{
    database::Database,
    io::{
        bus::{MessageBus, BusMessage},
        serial::{SspMessage, SourceInfo, Transport, MessageType},
    }
};
use crate::{log_info, log_warn, log_error};

// Survon BLE Service UUID (custom UUID for field units)
const SURVON_SERVICE_UUID: &str = "6e400001-b5a3-f393-e0a9-e50e24dcca9e";
const SURVON_TX_CHAR_UUID: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e"; // Write to device
const SURVON_RX_CHAR_UUID: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e"; // Notifications from device

// Compact SSP registration response (new format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompactRegistrationResponse {
    #[serde(rename = "p")]
    protocol: String,
    #[serde(rename = "t")]
    msg_type: String,
    #[serde(rename = "i")]
    device_id: String,
    #[serde(rename = "s")]
    timestamp: u64,
    #[serde(rename = "d")]
    data: CompactCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompactCapabilities {
    #[serde(rename = "dt")]
    device_type: String,
    #[serde(rename = "fw")]
    firmware: String,
    #[serde(rename = "s", default)]
    sensors: Vec<String>,        // Just keys: ["a", "b", "c"]
    #[serde(rename = "a", default)]
    actuators: Vec<String>,       // Just keys: ["led"]
}

impl CompactRegistrationResponse {
    fn to_capabilities(self) -> DeviceCapabilities {
        DeviceCapabilities {
            device_id: self.device_id,
            device_type: self.data.device_type,
            firmware_version: self.data.firmware,
            // Convert key arrays to SensorCapability structs
            sensors: self.data.sensors.iter().map(|key| SensorCapability {
                name: key.clone(),
                unit: "".to_string(),  // Unknown for compact format
                min_value: None,
                max_value: None,
            }).collect(),
            actuators: self.data.actuators.iter().map(|key| ActuatorCapability {
                name: key.clone(),
                actuator_type: "digital".to_string(),  // Default assumption
            }).collect(),
            commands: Vec::new(),
        }
    }
}

/// Device capabilities reported during registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub device_id: String,
    pub device_type: String,
    pub firmware_version: String,
    pub sensors: Vec<SensorCapability>,
    pub actuators: Vec<ActuatorCapability>,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCapability {
    pub name: String,
    pub unit: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorCapability {
    pub name: String,
    pub actuator_type: String, // "digital", "analog", "servo", etc.
}

/// Discovered but not yet registered device
#[derive(Debug, Clone)]
struct DiscoveredDevice {
    peripheral: Peripheral,
    name: String,
    address: String,
    rssi: i16,
}

/// Manages BLE device discovery and registration
#[derive(Debug, Clone)]
pub struct DiscoveryManager {
    adapter: Arc<RwLock<Option<Adapter>>>,
    discovered_devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    registered_devices: Arc<RwLock<HashMap<String, DeviceCapabilities>>>,
    message_bus: MessageBus,
    modules_path: std::path::PathBuf,
    database: Database,
}

impl DiscoveryManager {
    pub fn new(message_bus: MessageBus, modules_path: std::path::PathBuf, database: Database) -> Self {
        Self {
            adapter: Arc::new(RwLock::new(None)),
            discovered_devices: Arc::new(RwLock::new(HashMap::new())),
            registered_devices: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            modules_path,
            database,
        }
    }

    /// Start the discovery service
    pub async fn start(self: Arc<Self>) -> Result<()> {
        log_info!("Starting BLE Discovery Manager (manual scan mode)");

        // Initialize BLE adapter
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;

        if adapters.is_empty() {
            log_warn!("No BLE adapters found - discovery disabled");
            return Ok(());
        }

        let adapter = adapters.into_iter().next().unwrap();
        log_info!("Using BLE adapter: {}", adapter.adapter_info().await?);

        *self.adapter.write().await = Some(adapter.clone());

        log_info!("BLE Discovery Manager started (manual scan mode)");
        Ok(())
    }

    /// Perform a single scan cycle (10 seconds)
    /// Returns number of new Survon devices discovered
    pub async fn scan_once(&self, duration_secs: u64) -> Result<usize> {
        let adapter_lock = self.adapter.read().await;
        let adapter = adapter_lock
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("BLE adapter not initialized"))?;

        log_info!("🔍 Starting BLE scan ({} seconds)...", duration_secs);

        // Start scanning
        adapter.start_scan(ScanFilter::default()).await?;

        // CRITICAL: Scan for the requested duration
        // This is the blocking part - must be in separate tokio task
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;

        // Get peripherals BEFORE stopping scan (critical fix from original code)
        let peripherals = adapter.peripherals().await?;

        // Now safe to stop
        adapter.stop_scan().await?;

        log_info!("✅ Scan complete, processing {} peripheral(s)...", peripherals.len());

        let mut discovered_count = 0;

        for peripheral in peripherals {
            if let Some(properties) = peripheral.properties().await? {
                let name = properties
                    .local_name
                    .unwrap_or_else(|| "Unknown".to_string());

                // Check if this is a Survon device
                if name.contains("Survon") || name.contains("Field Unit") {
                    let address = properties.address.to_string();
                    let rssi = properties.rssi.unwrap_or(0);

                    log_info!("📡 Found: {} ({}) RSSI: {} dBm", name, address, rssi);

                    // Record in database
                    let is_new_device = self.database.record_device_discovery(
                        &address,
                        &name,
                        rssi
                    )?;

                    if is_new_device {
                        discovered_count += 1;
                    }

                    // Store in discovered devices map
                    let device = DiscoveredDevice {
                        peripheral: peripheral.clone(),
                        name: name.clone(),
                        address: address.clone(),
                        rssi,
                    };
                    self.discovered_devices.write().await.insert(address.clone(), device);

                    // Check trust status
                    let is_trusted = self.database.is_device_trusted(&address)?;

                    if is_trusted {
                        log_info!("✓ Device {} is trusted, attempting registration", address);

                        // Clone what we need before spawning
                        let periph = peripheral.clone();
                        let addr = address.clone();
                        let self_clone = Arc::new(self.clone());

                        // Register in background to avoid blocking scan results
                        tokio::spawn(async move {
                            if let Err(e) = self_clone.register_device(periph, addr.clone()).await {
                                log_error!("Failed to register trusted device {}: {}", addr, e);
                            }
                        });
                    } else if is_new_device {
                        log_info!("🆕 NEW device {} discovered, awaiting trust decision", address);

                        // Send trust prompt to UI
                        let event = BusMessage::new(
                            "device_discovered".to_string(),
                            serde_json::json!({
                            "mac_address": address,
                            "name": name,
                            "rssi": rssi,
                            "is_new": true,
                            "requires_trust_decision": true
                        }).to_string(),
                            "discovery_manager".to_string(),
                        );

                        if let Err(e) = self.message_bus.publish(event).await {
                            log_error!("Failed to publish device_discovered event: {}", e);
                        }
                    } else {
                        log_info!("Device {} is known but not trusted", address);
                    }
                }
            }
        }

        log_info!("✅ Scan complete - {} new Survon device(s) discovered", discovered_count);
        Ok(discovered_count)
    }

    /// Main scanning loop, disabled because it was blocking the app every 30 seconds for 10 seconds
    /// This is now only used if you want automatic background scanning
    async fn scan_loop(&self, adapter: Adapter) -> Result<()> {
        // This loop is now optional and should only be spawned if desired
        loop {
            if let Err(e) = self.scan_once(10).await {
                log_error!("Scan error: {}", e);
            }

            // Wait 30 seconds before next automatic scan
            tokio::time::sleep(Duration::from_secs(40)).await;
        }
    }

    // If you want to enable automatic scanning, add this method:
    pub async fn start_auto_scan(self: Arc<Self>) -> Result<()> {
        let adapter_lock = self.adapter.read().await;
        let adapter = adapter_lock
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("BLE adapter not initialized"))?
            .clone();
        drop(adapter_lock);

        let scanner = self.clone();
        tokio::spawn(async move {
            if let Err(e) = scanner.scan_loop(adapter).await {
                log_error!("Auto-scanner error: {}", e);
            }
        });

        log_info!("Automatic scanning enabled");
        Ok(())
    }

    /// Register a discovered device
    async fn register_device(&self, peripheral: Peripheral, address: String) -> Result<()> {
        log_info!("Connecting to device: {}", address);

        // Step 1: Connect to device
        peripheral.connect().await?;
        log_info!("✓ Connected to {}", address);

        // Step 2: Discover services
        peripheral.discover_services().await?;
        log_info!("✓ Discovered services for {}", address);

        // Step 3: Find the RX characteristic (notifications from device)
        let chars = peripheral.characteristics();
        let rx_char = chars.iter()
            .find(|c| c.uuid == Uuid::parse_str(SURVON_RX_CHAR_UUID).unwrap())
            .ok_or_else(|| color_eyre::eyre::eyre!("RX characteristic not found"))?;

        log_info!("✓ Found RX characteristic");

        // Step 4: Subscribe to notifications
        peripheral.subscribe(rx_char).await?;
        log_info!("✓ Subscribed to notifications from {}", address);

        // Step 5: Find the TX characteristic (write to device)
        let tx_char = chars.iter()
            .find(|c| c.uuid == Uuid::parse_str(SURVON_TX_CHAR_UUID).unwrap())
            .ok_or_else(|| color_eyre::eyre::eyre!("TX characteristic not found"))?;

        log_info!("✓ Found TX characteristic");

        // Step 6: Create a channel to receive the registration response
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel::<DeviceCapabilities>(1);

        // Step 7: Spawn listener for incoming data
        let bus = self.message_bus.clone();
        let periph = peripheral.clone();
        let addr_clone = address.clone();
        let rx_char_clone = rx_char.clone();

        // Hybrid approach: Event-driven with keepalive
        // Best of both worlds - no constant polling, but prevents timeout

        tokio::spawn(async move {
            log_info!("📷 BLE listener task started for {}", addr_clone);

            let mut registration_sent = false;

            // Outer loop: handles reconnection if stream dies
            loop {
                log_info!("📡 Acquiring notification stream for {}...", addr_clone);

                match periph.notifications().await {
                    Ok(mut stream) => {
                        log_info!("✅ Got notification stream for {}", addr_clone);

                        let mut buffer = String::new();
                        let mut last_chunk_time = std::time::Instant::now();
                        let mut chunk_count = 0;

                        // Event-driven with occasional keepalive check
                        loop {
                            // Wait up to 15 seconds for data (your Arduino sends every 3s)
                            // This gives plenty of margin without constant polling
                            match tokio::time::timeout(
                                Duration::from_secs(15),
                                stream.next()
                            ).await {
                                Ok(Some(data)) => {
                                    // Got data! Process it normally
                                    chunk_count += 1;
                                    let chunk = String::from_utf8_lossy(&data.value).to_string();

                                    log_info!("📦 Chunk #{}: {} bytes", chunk_count, chunk.len());

                                    // If more than 500ms since last chunk, assume new message
                                    if last_chunk_time.elapsed().as_millis() > 500 {
                                        if !buffer.is_empty() {
                                            log_warn!("⚠️ Timeout - clearing incomplete buffer");
                                        }
                                        buffer.clear();
                                    }

                                    buffer.push_str(&chunk);
                                    last_chunk_time = std::time::Instant::now();

                                    // Check if complete JSON
                                    if buffer.starts_with('{') && buffer.ends_with('}') {
                                        log_info!("✅ COMPLETE MESSAGE ({} bytes)", buffer.len());

                                        // Try compact format first
                                        if buffer.contains("\"dt\"") && buffer.contains("\"fw\"") && !registration_sent {
                                            log_info!("📷 Detected COMPACT registration format");

                                            match serde_json::from_str::<CompactRegistrationResponse>(&buffer) {
                                                Ok(compact_reg) => {
                                                    log_info!("✅ Parsed compact registration");
                                                    let capabilities = compact_reg.to_capabilities();

                                                    if response_tx.send(capabilities).await.is_ok() {
                                                        log_info!("✅ Sent registration response to handler");
                                                        registration_sent = true;
                                                    }
                                                }
                                                Err(e) => {
                                                    log_error!("❌ Failed to parse compact registration: {}", e);
                                                }
                                            }
                                        }
                                        // Try verbose format
                                        else if buffer.contains("\"device_id\"") && !registration_sent {
                                            log_info!("📋 Detected VERBOSE registration format");

                                            match serde_json::from_str::<DeviceCapabilities>(&buffer) {
                                                Ok(capabilities) => {
                                                    log_info!("✅ Parsed verbose registration");

                                                    if response_tx.send(capabilities).await.is_ok() {
                                                        log_info!("✅ Sent registration response to handler");
                                                        registration_sent = true;
                                                    }
                                                }
                                                Err(e) => {
                                                    log_error!("❌ Failed to parse verbose registration: {}", e);
                                                }
                                            }
                                        }
                                        // Regular telemetry
                                        else {
                                            log_info!("📊 Attempting SSP telemetry parse...");
                                            match SspMessage::parse_flexible(&buffer) {
                                                Ok(ssp) => {
                                                    log_info!("✅ Parsed SSP telemetry - topic: {}", ssp.topic);
                                                    let bus_msg = ssp.to_bus_message();
                                                    match bus.publish(bus_msg).await {
                                                        Ok(_) => log_info!("✅ Published to bus"),
                                                        Err(e) => log_error!("❌ Publish failed: {}", e),
                                                    }
                                                }
                                                Err(_) => {
                                                    log_warn!("⚠️ Not a recognized message format");
                                                    log_warn!("Buffer content: {}", buffer);
                                                }
                                            }
                                        }

                                        buffer.clear();
                                    }
                                }
                                Ok(None) => {
                                    // Stream ended naturally
                                    log_warn!("📡 Notification stream ended for {} (disconnect)", addr_clone);
                                    break;
                                }
                                Err(_) => {
                                    // Timeout: No data for 15 seconds (should never happen with 3s telemetry!)
                                    log_warn!("⚠️ No data from {} for 15 seconds - checking connection", addr_clone);

                                    // Check if still connected
                                    if !periph.is_connected().await.unwrap_or(false) {
                                        log_error!("❌ Device {} disconnected", addr_clone);
                                        break;
                                    }

                                    // Still connected but no data - this is suspicious
                                    // Could mean the Arduino stopped sending or BLE link is flaky
                                    log_warn!("⚠️ Device {} still connected but silent - continuing to listen", addr_clone);
                                }
                            }
                        }

                        log_info!("🔄 Stream closed for {}, will attempt reconnection in 5s...", addr_clone);
                    }
                    Err(e) => {
                        log_error!("❌ Failed to get notification stream for {}: {}", addr_clone, e);
                    }
                }

                // Wait before attempting reconnection
                tokio::time::sleep(Duration::from_secs(5)).await;

                // Check if device is disconnected and try to reconnect
                if !periph.is_connected().await.unwrap_or(false) {
                    log_info!("🔌 Device {} not connected, attempting to reconnect...", addr_clone);

                    match periph.connect().await {
                        Ok(_) => {
                            log_info!("✅ Reconnected to {}", addr_clone);

                            // Re-discover services
                            if let Err(e) = periph.discover_services().await {
                                log_error!("❌ Failed to rediscover services: {}", e);
                                continue;
                            }

                            // Re-subscribe to notifications
                            if let Err(e) = periph.subscribe(&rx_char_clone).await {
                                log_error!("❌ Failed to resubscribe: {}", e);
                                continue;
                            }

                            log_info!("✅ Resubscribed to notifications for {}", addr_clone);
                        }
                        Err(e) => {
                            log_error!("❌ Failed to reconnect to {}: {}", addr_clone, e);
                        }
                    }
                } else {
                    log_info!("🔄 Device {} still connected, reacquiring stream...", addr_clone);
                }
            }
        });

        // Step 8: Send registration request
        log_info!("Sending registration request to device...");

        let registration_request = serde_json::json!({
            "hub_id": "survon_hub",
            "request": "capabilities",
            "timestamp": chrono::Utc::now().timestamp() as u64
        });

        let request_bytes = registration_request.to_string().into_bytes();
        peripheral.write(tx_char, &request_bytes, WriteType::WithoutResponse).await?;

        log_info!("✓ Sent registration request");

        // Step 9: Wait for registration response (with timeout)
        log_info!("Waiting for device capabilities response...");

        match timeout(Duration::from_secs(10), response_rx.recv()).await {
            Ok(Some(capabilities)) => {
                log_info!("✓ Received device capabilities!");

                // Step 10: Complete registration
                self.handle_registration(capabilities).await?;

                Ok(())
            }
            Ok(None) => {
                log_error!("Registration channel closed unexpectedly");
                Err(color_eyre::eyre::eyre!("Registration channel closed"))
            }
            Err(_) => {
                log_error!("Timeout waiting for device registration response");
                Err(color_eyre::eyre::eyre!("Device did not respond to registration request"))
            }
        }
    }

    async fn get_fresh_peripheral_for_reconnect(
        adapter_lock: &Arc<RwLock<Option<Adapter>>>,
        address: &str,
    ) -> Result<Peripheral> {
        let adapter = adapter_lock.read().await;
        let adapter = adapter.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Adapter not available"))?;

        log_info!("🔍 Rescanning to get fresh peripheral for {}", address);

        // Quick 2-second scan to refresh peripheral list
        adapter.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        adapter.stop_scan().await?;

        // Find our device in the FRESH peripheral list
        let peripherals = adapter.peripherals().await?;

        for periph in peripherals {
            if let Ok(Some(props)) = periph.properties().await {
                if props.address.to_string() == address {
                    log_info!("✓ Found fresh peripheral for {}", address);
                    return Ok(periph);
                }
            }
        }

        Err(color_eyre::eyre::eyre!("Device not found in rescan"))
    }

    /// Handle successful registration
    async fn handle_registration(&self, capabilities: DeviceCapabilities) -> Result<()> {
        log_info!("Registering device: {} ({})", capabilities.device_id, capabilities.device_type);

        // Store in registered devices
        self.registered_devices.write().await.insert(
            capabilities.device_id.clone(),
            capabilities.clone(),
        );

        // Generate module YAML
        self.generate_module_config(&capabilities).await?;

        // Publish registration event to message bus
        let event = BusMessage::new(
            "device_registered".to_string(),
            serde_json::to_string(&capabilities)?,
            "discovery_manager".to_string(),
        );
        self.message_bus.publish(event).await?;

        log_info!("✓ Device {} registered successfully", capabilities.device_id);

        Ok(())
    }

    /// Check if a device is trusted
    async fn is_trusted(&self, mac_address: &str) -> Result<bool> {
        // Query database for trusted status
        let result = self.database.is_device_trusted(mac_address)?;
        Ok(result)
    }

    /// Trust a device (called from UI)
    pub async fn trust_device(&self, mac_address: String) -> Result<()> {
        log_info!("Trusting device: {}", mac_address);

        // Get device name from discovered devices
        let device_name = {
            let devices = self.discovered_devices.read().await;
            devices.get(&mac_address)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Unknown Device".to_string())
        };

        // Store in database
        self.database.trust_device(&mac_address, &device_name)?;

        // Attempt registration
        if let Some(device) = self.discovered_devices.read().await.get(&mac_address) {
            let peripheral = device.peripheral.clone();
            self.register_device(peripheral, mac_address.clone()).await?;
        }

        Ok(())
    }

    /// Untrust a device
    pub async fn untrust_device(&self, mac_address: &str) -> Result<()> {
        log_info!("Untrusting device: {}", mac_address);
        self.database.untrust_device(mac_address)?;
        Ok(())
    }

    /// Get all trusted devices from database
    pub async fn get_trusted_devices(&self) -> Result<Vec<(String, String)>> {
        Ok(self.database.get_trusted_devices()?)
    }

    /// Generate module config.yml for the registered device
    async fn generate_module_config(&self, capabilities: &DeviceCapabilities) -> Result<()> {
        let module_path = self.modules_path.join(&capabilities.device_id);
        std::fs::create_dir_all(&module_path)?;

        let config_path = module_path.join("config.yml");

        // Determine template based on device type and capabilities
        let template = self.select_template(capabilities);

        // Generate bindings based on sensors
        let mut bindings = serde_yaml::Mapping::new();

        for sensor in &capabilities.sensors {
            bindings.insert(
                serde_yaml::Value::String(sensor.name.clone()),
                serde_yaml::Value::Number(serde_yaml::Number::from(0)),
            );
        }

        // Add standard bindings
        bindings.insert(
            serde_yaml::Value::String("device_id".to_string()),
            serde_yaml::Value::String(capabilities.device_id.clone()),
        );
        bindings.insert(
            serde_yaml::Value::String("firmware_version".to_string()),
            serde_yaml::Value::String(capabilities.firmware_version.clone()),
        );
        bindings.insert(
            serde_yaml::Value::String("is_blinkable".to_string()),
            serde_yaml::Value::Bool(true),
        );

        // Build config structure
        let mut config = serde_yaml::Mapping::new();
        config.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String(format!("{} ({})", capabilities.device_id, capabilities.device_type)),
        );
        config.insert(
            serde_yaml::Value::String("module_type".to_string()),
            serde_yaml::Value::String("monitoring".to_string()),
        );
        config.insert(
            serde_yaml::Value::String("bus_topic".to_string()),
            serde_yaml::Value::String(capabilities.device_id.clone()),
        );
        config.insert(
            serde_yaml::Value::String("template".to_string()),
            serde_yaml::Value::String(template),
        );
        config.insert(
            serde_yaml::Value::String("bindings".to_string()),
            serde_yaml::Value::Mapping(bindings),
        );

        // Generate sample SSP payloads in comments
        let sample_payload = self.generate_sample_payload(capabilities);

        // Write to file
        let yaml_content = serde_yaml::to_string(&config)?;
        let full_content = format!(
            "# Auto-generated module for {}\n# Device Type: {}\n# Firmware: {}\n\n# Sample SSP Telemetry Payload:\n{}\n\n{}",
            capabilities.device_id,
            capabilities.device_type,
            capabilities.firmware_version,
            sample_payload,
            yaml_content
        );

        std::fs::write(config_path, full_content)?;

        log_info!("Generated module config at: {}", module_path.display());

        Ok(())
    }

    /// Select appropriate template based on capabilities
    fn select_template(&self, capabilities: &DeviceCapabilities) -> String {
        // If device has actuators, use control template
        if !capabilities.actuators.is_empty() {
            return "toggle_switch".to_string();
        }

        // If device has multiple sensors, use status badge or activity card
        if capabilities.sensors.len() > 1 {
            return "status_badge_card".to_string();
        }

        // Single sensor - use gauge
        if capabilities.sensors.len() == 1 {
            let sensor = &capabilities.sensors[0];
            if sensor.max_value.is_some() {
                return "gauge_card".to_string();
            }
        }

        // Default to status badge
        "status_badge_card".to_string()
    }

    /// Generate sample SSP payload documentation
    fn generate_sample_payload(&self, capabilities: &DeviceCapabilities) -> String {
        let mut payload = serde_json::Map::new();

        for sensor in &capabilities.sensors {
            payload.insert(
                sensor.name.clone(),
                serde_json::json!(sensor.min_value.unwrap_or(0.0)),
            );
        }

        let sample = serde_json::json!({
            "protocol": "ssp/1.0",
            "type": "telemetry",
            "topic": capabilities.device_id,
            "timestamp": 1732377600u64,
            "source": {
                "id": capabilities.device_id,
                "transport": "ble",
                "address": "XX:XX:XX:XX:XX:XX"
            },
            "payload": payload
        });

        format!("# {}", serde_json::to_string_pretty(&sample).unwrap()
            .lines()
            .collect::<Vec<_>>()
            .join("\n# "))
    }

    /// Get list of discovered but unregistered devices
    pub async fn get_discovered_devices(&self) -> Vec<(String, String, i16)> {
        self.discovered_devices
            .read()
            .await
            .values()
            .map(|d| (d.address.clone(), d.name.clone(), d.rssi))
            .collect()
    }

    /// Get list of registered devices
    pub async fn get_registered_devices(&self) -> Vec<DeviceCapabilities> {
        self.registered_devices
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }
}

