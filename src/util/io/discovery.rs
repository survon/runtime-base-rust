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
        ble_scheduler::{BleCommandScheduler, QueuedCommand, CommandPriority, extract_schedule_metadata},
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
    command_scheduler: Arc<BleCommandScheduler>,
}

impl DiscoveryManager {
    pub fn new(message_bus: MessageBus, modules_path: std::path::PathBuf, database: Database) -> Self {
        let command_scheduler = Arc::new(
            BleCommandScheduler::new().with_message_bus(message_bus.clone())
        );

        Self {
            adapter: Arc::new(RwLock::new(None)),
            discovered_devices: Arc::new(RwLock::new(HashMap::new())),
            registered_devices: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            modules_path,
            database,
            command_scheduler,
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

    pub async fn send_command(
        &self,
        device_id: String,
        action: &str,
        payload: Option<serde_json::Value>,
        priority: CommandPriority,
    ) -> Result<()> {
        log_info!("🎯 Command request: {} -> {}", device_id, action);

        let command = crate::util::io::ble_scheduler::create_control_command(
            &device_id,
            action,
            payload,
        );

        let queued_cmd = QueuedCommand {
            device_id: device_id.clone(),
            command,
            priority,
            queued_at: tokio::time::Instant::now(),
            max_age: Some(Duration::from_secs(300)),  // Commands expire after 5 minutes
        };

        self.command_scheduler.queue_command(queued_cmd).await?;

        Ok(())
    }

    /// Register a discovered device
    async fn register_device(&self, peripheral: Peripheral, address: String) -> Result<()> {
        log_info!("Connecting to device: {}", address);

        peripheral.connect().await?;
        log_info!("✓ Connected to {}", address);

        // Small delay for CoreBluetooth to stabilize
        tokio::time::sleep(Duration::from_millis(500)).await;

        peripheral.discover_services().await?;
        log_info!("✓ Discovered services for {}", address);

        let chars = peripheral.characteristics();
        let rx_char = chars.iter()
            .find(|c| c.uuid == Uuid::parse_str(SURVON_RX_CHAR_UUID).unwrap())
            .ok_or_else(|| color_eyre::eyre::eyre!("RX characteristic not found"))?;

        peripheral.subscribe(rx_char).await?;
        log_info!("✓ Subscribed to notifications from {}", address);

        let tx_char = chars.iter()
            .find(|c| c.uuid == Uuid::parse_str(SURVON_TX_CHAR_UUID).unwrap())
            .ok_or_else(|| color_eyre::eyre::eyre!("TX characteristic not found"))?;

        // 🔑 KEY CHANGE: No registration channel - we'll auto-register from telemetry

        // Register peripheral with scheduler immediately
        self.command_scheduler.register_peripheral(
            address.clone(),
            peripheral.clone()
        ).await;

        // Spawn listener for incoming data
        let bus = self.message_bus.clone();
        let addr_clone = address.clone();
        let rx_char_clone = rx_char.clone();
        let scheduler = self.command_scheduler.clone();
        let adapter_lock = self.adapter.clone();
        let peripheral_clone = peripheral.clone();

        // 🔑 NEW: Clone self for auto-registration
        let self_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            log_info!("📻 BLE listener task started for {}", addr_clone);

            let mut current_peripheral = peripheral_clone;
            let mut device_registered = false;  // Track if we've registered capabilities

            loop {
                log_info!("📡 Acquiring notification stream for {}...", addr_clone);

                match current_peripheral.notifications().await {
                    Ok(mut stream) => {
                        log_info!("✅ Got notification stream for {}", addr_clone);

                        // 🫀 KEEP-ALIVE TASK
                        let keepalive_peripheral = current_peripheral.clone();
                        let keepalive_char = rx_char_clone.clone();
                        let keepalive_addr = addr_clone.clone();

                        let keepalive_handle = tokio::spawn(async move {
                            let mut iteration = 0;
                            loop {
                                tokio::time::sleep(Duration::from_secs(10)).await;
                                iteration += 1;

                                log_info!("🫀 Keep-alive iteration {} for {}", iteration, keepalive_addr);

                                match keepalive_peripheral.read(&keepalive_char).await {
                                    Ok(data) => {
                                        log_info!("🫀 Keep-alive read OK for {} (got {} bytes)", keepalive_addr, data.len());
                                    }
                                    Err(e) => {
                                        log_warn!("🫀 Keep-alive failed for {}: {}", keepalive_addr, e);
                                        break;
                                    }
                                }
                            }
                        });

                        let mut buffer = String::new();
                        let mut last_chunk_time = std::time::Instant::now();

                        loop {
                            match tokio::time::timeout(
                                Duration::from_secs(5),
                                stream.next()
                            ).await {
                                Ok(Some(data)) => {
                                    log_info!("📥 Received {} bytes from {}", data.value.len(), addr_clone);

                                    let chunk = String::from_utf8_lossy(&data.value).to_string();

                                    // Clear stale buffers
                                    if last_chunk_time.elapsed().as_secs() > 3 {
                                        if !buffer.is_empty() {
                                            log_warn!("⚠️ Clearing stale buffer ({} bytes)", buffer.len());
                                            buffer.clear();
                                        }
                                    }

                                    buffer.push_str(&chunk);
                                    last_chunk_time = std::time::Instant::now();

                                    log_info!("📝 Buffer now {} bytes", buffer.len());

                                    // 🔑 Process ALL complete messages (delimited by newlines)
                                    while let Some(newline_pos) = buffer.find('\n') {
                                        let message = buffer[..newline_pos].to_string();
                                        buffer = buffer[newline_pos + 1..].to_string();

                                        let message = message.trim();

                                        if message.is_empty() {
                                            continue;
                                        }

                                        // Validate JSON structure
                                        if !message.starts_with('{') {
                                            log_warn!("⚠️ Skipping malformed message (doesn't start with '{{' ): {}", message);
                                            continue;
                                        }

                                        log_info!("✅ COMPLETE MESSAGE ({} bytes): {}", message.len(), message);

                                        // 🔑 AUTO-REGISTRATION: First telemetry message triggers registration
                                        if !device_registered {
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                                                if let Some(device_id) = json.get("i").and_then(|v| v.as_str()) {
                                                    log_info!("🆔 Discovered device ID: {}", device_id);

                                                    let capabilities = DeviceCapabilities {
                                                        device_id: device_id.to_string(),
                                                        device_type: "field_unit".to_string(),
                                                        firmware_version: "unknown".to_string(),
                                                        sensors: vec![
                                                            SensorCapability {
                                                                name: "a".to_string(),
                                                                unit: "".to_string(),
                                                                min_value: None,
                                                                max_value: None,
                                                            },
                                                            SensorCapability {
                                                                name: "b".to_string(),
                                                                unit: "".to_string(),
                                                                min_value: None,
                                                                max_value: None,
                                                            },
                                                            SensorCapability {
                                                                name: "c".to_string(),
                                                                unit: "".to_string(),
                                                                min_value: None,
                                                                max_value: None,
                                                            },
                                                        ],
                                                        actuators: Vec::new(),
                                                        commands: Vec::new(),
                                                    };

                                                    if let Err(e) = self_clone.handle_registration(capabilities).await {
                                                        log_error!("Failed to register device: {}", e);
                                                    } else {
                                                        log_info!("✅ Device {} auto-registered from telemetry", device_id);
                                                        device_registered = true;
                                                    }
                                                }
                                            }
                                        }

                                        // Parse as telemetry
                                        match SspMessage::parse_flexible(&message) {
                                            Ok(ssp) => {
                                                log_info!("✅ Parsed SSP telemetry - topic: {}", ssp.topic);

                                                // Extract schedule metadata
                                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                                                    if let Some(metadata) = extract_schedule_metadata(&json) {
                                                        if let Err(e) = scheduler.update_schedule_from_telemetry(
                                                            addr_clone.clone(),
                                                            &metadata
                                                        ).await {
                                                            log_error!("Failed to update schedule: {}", e);
                                                        }
                                                    }
                                                }

                                                // Publish to bus
                                                let bus_msg = ssp.to_bus_message();
                                                match bus.publish(bus_msg).await {
                                                    Ok(_) => log_info!("✅ Published to bus"),
                                                    Err(e) => log_error!("❌ Publish failed: {}", e),
                                                }
                                            }
                                            Err(e) => {
                                                log_warn!("⚠️ Failed to parse as SSP: {}", e);
                                            }
                                        }
                                    }

                                    // Log remaining buffer state
                                    if !buffer.is_empty() {
                                        log_info!("⏳ {} bytes remaining in buffer (incomplete message)", buffer.len());
                                    }
                                }
                                Ok(None) => {
                                    log_warn!("📡 Stream returned None");

                                    let is_connected = current_peripheral.is_connected().await.unwrap_or(false);
                                    log_info!("🔌 Connection state: {}", if is_connected { "CONNECTED" } else { "DISCONNECTED" });

                                    keepalive_handle.abort();

                                    if let Err(e) = Self::handle_reconnect(
                                        &adapter_lock,
                                        &addr_clone,
                                        &rx_char_clone,
                                        &mut current_peripheral,
                                        &scheduler
                                    ).await {
                                        log_error!("❌ Reconnect failed: {}", e);
                                        tokio::time::sleep(Duration::from_secs(5)).await;
                                    }

                                    break;
                                }
                                Err(_) => {
                                    log_warn!("⏰ Timeout waiting for chunk (buffer: {} bytes)", buffer.len());

                                    if !buffer.is_empty() {
                                        log_warn!("⚠️ Clearing incomplete message");
                                        buffer.clear();
                                    }

                                    let is_connected = current_peripheral.is_connected().await.unwrap_or(false);

                                    if !is_connected {
                                        log_error!("📡 Device disconnected during timeout");
                                        keepalive_handle.abort();

                                        if let Err(e) = Self::handle_reconnect(
                                            &adapter_lock,
                                            &addr_clone,
                                            &rx_char_clone,
                                            &mut current_peripheral,
                                            &scheduler
                                        ).await {
                                            log_error!("❌ Reconnect failed: {}", e);
                                            tokio::time::sleep(Duration::from_secs(5)).await;
                                        }

                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log_error!("❌ Failed to get notification stream: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }

                log_info!("🔄 Restarting stream loop for {}...", addr_clone);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        // 🔑 KEY CHANGE: Return immediately, don't wait for registration
        log_info!("✅ Listener spawned for {}, will auto-register from telemetry", address);

        Ok(())
    }

    fn extract_one_json_message(buffer: &str) -> Result<(String, String), String> {
        let start = buffer.find('{').ok_or("No JSON start found")?;

        let mut depth = 0;
        let mut end = None;

        for (i, ch) in buffer[start..].chars().enumerate() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }

        let end = end.ok_or("No matching closing brace")?;

        let json_message = buffer[start..end].to_string();
        let remaining = if end < buffer.len() {
            buffer[end..].to_string()
        } else {
            String::new()
        };

        Ok((json_message, remaining))
    }

    async fn handle_reconnect(
        adapter_lock: &Arc<RwLock<Option<Adapter>>>,
        address: &str,
        rx_char: &btleplug::api::Characteristic,
        current_peripheral: &mut Peripheral,
        scheduler: &Arc<BleCommandScheduler>,
    ) -> Result<()> {
        log_info!("🔄 Starting reconnect for {}", address);

        let adapter = adapter_lock.read().await;
        let adapter = adapter.as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Adapter not available"))?;

        // Quick 3-second scan to refresh peripheral list
        log_info!("🔍 Rescanning for {}", address);
        adapter.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        let peripherals = adapter.peripherals().await?;
        adapter.stop_scan().await?;

        // Find device in fresh peripheral list
        for periph in peripherals {
            if let Ok(Some(props)) = periph.properties().await {
                if props.address.to_string() == address {
                    log_info!("✅ Found peripheral for {}", address);

                    // Connect
                    periph.connect().await?;
                    log_info!("✅ Connected to {}", address);

                    // Small delay for CoreBluetooth to stabilize
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    // Discover services
                    periph.discover_services().await?;
                    log_info!("✅ Services discovered for {}", address);

                    // Subscribe to notifications
                    periph.subscribe(rx_char).await?;
                    log_info!("✅ Subscribed to notifications for {}", address);

                    // Update peripheral reference
                    *current_peripheral = periph.clone();

                    // Re-register with scheduler
                    scheduler.register_peripheral(
                        address.to_string(),
                        periph
                    ).await;

                    log_info!("✅ Reconnect complete for {}", address);
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!("Device {} not found in rescan", address))
    }

    /// Connect to all trusted devices from database
    pub async fn connect_trusted_devices(&self) -> Result<()> {
        log_info!("🔍 Connecting to trusted devices from database...");

        let trusted = self.database.get_trusted_devices()?;

        if trusted.is_empty() {
            log_info!("No trusted devices in database");
            return Ok(());
        }

        log_info!("Found {} trusted device(s) in database", trusted.len());

        // Do a quick scan to find peripherals
        log_info!("📡 Scanning for trusted devices...");
        let found = self.scan_once(10).await?;
        log_info!("Scan found {} total device(s)", found);

        // Now try to connect to each trusted device
        let discovered = self.discovered_devices.read().await;
        let mut connected_count = 0;

        for (mac, name) in trusted {
            if let Some(device) = discovered.get(&mac) {
                log_info!("✅ Found trusted device: {} ({})", name, mac);

                let peripheral = device.peripheral.clone();
                let mac_clone = mac.clone();
                let self_clone = Arc::new(self.clone());

                // Register in background
                tokio::spawn(async move {
                    match self_clone.register_device(peripheral, mac_clone.clone()).await {
                        Ok(_) => log_info!("✅ Registered trusted device: {}", mac_clone),
                        Err(e) => log_error!("❌ Failed to register {}: {}", mac_clone, e),
                    }
                });

                connected_count += 1;
            } else {
                log_warn!("⚠️ Trusted device {} ({}) not found in scan", name, mac);
            }
        }

        log_info!("🔌 Initiated connection to {} trusted device(s)", connected_count);

        Ok(())
    }

    pub fn get_scheduler(&self) -> Arc<BleCommandScheduler> {
        self.command_scheduler.clone()
    }

    pub async fn start_maintenance_task(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;

                // Prune stale schedules
                self.command_scheduler.prune_stale_schedules().await;
            }
        });
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

