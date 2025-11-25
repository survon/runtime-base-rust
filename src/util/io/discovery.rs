// src/util/io/discovery.rs
//! BLE Field Unit Discovery - Auto-detect and register Survon-compatible devices

use color_eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
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
        log_info!("Starting BLE Discovery Manager");

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

        // Spawn scanner task
        let scanner = self.clone();
        tokio::spawn(async move {
            if let Err(e) = scanner.scan_loop(adapter).await {
                log_error!("Scanner error: {}", e);
            }
        });

        log_info!("BLE Discovery Manager started");
        Ok(())
    }

    /// Main scanning loop
    async fn scan_loop(&self, adapter: Adapter) -> Result<()> {
        let mut events = adapter.events().await?;

        loop {
            log_info!("Scanning for Survon field units...");
            adapter.start_scan(ScanFilter::default()).await?;

            tokio::time::sleep(Duration::from_secs(10)).await;
            adapter.stop_scan().await?;

            let peripherals = adapter.peripherals().await?;

            for peripheral in peripherals {
                if let Some(properties) = peripheral.properties().await? {
                    let name = properties
                        .local_name
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Check if this is a Survon device
                    if name.contains("Survon") || name.contains("Field Unit") {
                        let address = properties.address.to_string();
                        let rssi = properties.rssi.unwrap_or(0);

                        log_info!("Found Survon device: {} ({}), RSSI: {}", name, address, rssi);

                        // Store as discovered
                        let device = DiscoveredDevice {
                            peripheral: peripheral.clone(),
                            name: name.clone(),
                            address: address.clone(),
                            rssi,
                        };

                        self.discovered_devices.write().await.insert(address.clone(), device);

                        // Only auto-register if trusted
                        if self.is_trusted(&address).await? {
                            log_info!("Device {} is trusted, attempting registration", address);
                            if let Err(e) = self.register_device(peripheral, address.clone()).await {
                                log_error!("Failed to register trusted device: {}", e);
                            }
                        } else {
                            log_info!("Device {} awaiting trust approval", address);

                            // Publish discovery event to message bus for UI notification
                            let event = BusMessage::new(
                                "device_discovered".to_string(),
                                serde_json::json!({
                                "mac_address": address,
                                "name": name,
                                "rssi": rssi
                            }).to_string(),
                                "discovery_manager".to_string(),
                            );
                            let _ = self.message_bus.publish(event).await;
                        }
                    }
                }
            }

            // Wait before next scan cycle
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    /// Register a discovered device
    async fn register_device(&self, peripheral: Peripheral, address: String) -> Result<()> {
        log_info!("Attempting to register device: {}", address);

        // Connect to the device
        if !peripheral.is_connected().await? {
            peripheral.connect().await?;
            log_info!("Connected to {}", address);
        }

        // Discover services
        peripheral.discover_services().await?;

        // Find Survon service and characteristics
        let chars = peripheral.characteristics();
        let tx_char = chars.iter().find(|c| {
            c.uuid == Uuid::parse_str(SURVON_TX_CHAR_UUID).unwrap()
        });
        let rx_char = chars.iter().find(|c| {
            c.uuid == Uuid::parse_str(SURVON_RX_CHAR_UUID).unwrap()
        });

        if tx_char.is_none() || rx_char.is_none() {
            log_warn!("Device {} doesn't have Survon characteristics", address);
            return Ok(());
        }

        let tx_char = tx_char.unwrap();
        let rx_char = rx_char.unwrap();

        // Subscribe to notifications
        peripheral.subscribe(rx_char).await?;

        // Send registration request
        let registration_request = SspMessage {
            protocol: "ssp/1.0".to_string(),
            msg_type: MessageType::Command,
            topic: "device_registration".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            source: SourceInfo {
                id: "survon_hub".to_string(),
                transport: Transport::Internal,
                address: "internal".to_string(),
            },
            payload: serde_json::json!({
                "action": "register",
                "request_capabilities": true
            }),
            qos: None,
            retain: None,
            reply_to: Some("survon_hub".to_string()),
            in_reply_to: None,
        };

        let request_json = serde_json::to_string(&registration_request)?;
        peripheral.write(tx_char, request_json.as_bytes(), WriteType::WithResponse).await?;

        log_info!("Sent registration request to {}", address);

        // Wait for response (with timeout)
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut notification_stream = peripheral.notifications().await?;

            while let Some(notification) = notification_stream.next().await {
                if notification.uuid == rx_char.uuid {
                    let response = String::from_utf8_lossy(&notification.value);
                    log_info!("Received response: {}", response);

                    // Parse capabilities
                    if let Ok(ssp_msg) = SspMessage::parse(&response) {
                        if ssp_msg.topic == "device_registration" {
                            if let Ok(capabilities) = serde_json::from_value::<DeviceCapabilities>(ssp_msg.payload) {
                                self.handle_registration(capabilities).await?;
                                return Ok::<(), color_eyre::Report>(());
                            }
                        }
                    }
                }
            }

            Err(color_eyre::eyre::eyre!("No valid registration response"))
        })
            .await
            .map_err(|_| color_eyre::eyre::eyre!("Registration timeout"))??;

        Ok(())
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

