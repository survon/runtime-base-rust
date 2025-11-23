// src/util/io/transport.rs
//! Transport Manager - Handles bidirectional communication with IoT devices

use color_eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::SerialPortBuilderExt;

use crate::util::io::{
    bus::{BusMessage, BusReceiver, MessageBus},
    serial::{SspMessage, SourceInfo, Transport, MessageType},
};
use crate::{log_info, log_warn, log_error};

/// Manages all transport connections and message routing
#[derive(Clone)]
pub struct TransportManager {
    /// Routing table: device_id -> source info (transport type + address)
    routing_table: Arc<RwLock<HashMap<String, SourceInfo>>>,
    /// Message bus for publishing inbound messages
    message_bus: MessageBus,
    /// Topics that should be forwarded to external devices
    outbound_topics: Arc<RwLock<Vec<String>>>,
}

impl TransportManager {
    pub fn new(message_bus: MessageBus) -> Self {
        Self {
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            outbound_topics: Arc::new(RwLock::new(vec![
                "com_input".to_string(),      // Gate control, etc.
                "control".to_string(),         // General control commands
                "arduino_ping".to_string(),    // Arduino ping messages
            ])),
        }
    }

    /// Add a topic that should be forwarded to external devices
    pub async fn add_outbound_topic(&self, topic: String) {
        let mut topics = self.outbound_topics.write().await;
        if !topics.contains(&topic) {
            topics.push(topic);
        }
    }

    /// Start the transport manager tasks
    pub async fn start(self) -> Result<()> {
        log_info!("Starting Transport Manager");

        // Subscribe to outbound topics
        let outbound_topics = self.outbound_topics.read().await.clone();
        let mut receivers = Vec::new();

        for topic in &outbound_topics {
            let receiver = self.message_bus.subscribe(topic.clone()).await;
            receivers.push(receiver);
            log_info!("Transport Manager subscribed to outbound topic: {}", topic);
        }

        // Clone for the outbound handler
        let manager_clone = self.clone();

        // Spawn outbound message handler
        tokio::spawn(async move {
            manager_clone.handle_outbound_messages(receivers).await;
        });

        // Start listening on available transports
        self.start_usb_listener().await?;

        log_info!("Transport Manager started");
        Ok(())
    }

    /// Handle messages from the bus that need to be sent to external devices
    async fn handle_outbound_messages(&self, mut receivers: Vec<BusReceiver>) {
        log_info!("Outbound message handler started");

        loop {
            // Poll all receivers
            for receiver in &mut receivers {
                while let Ok(bus_msg) = receiver.try_recv() {
                    log_info!("Outbound message on topic '{}': {}", bus_msg.topic, bus_msg.payload);

                    // Parse payload to determine target device
                    if let Err(e) = self.route_outbound_message(&bus_msg).await {
                        log_error!("Failed to route outbound message: {}", e);
                    }
                }
            }

            // Small delay to avoid busy-waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Route an outbound message to the appropriate device
    async fn route_outbound_message(&self, bus_msg: &BusMessage) -> Result<()> {
        // Try to extract target device from payload
        let target_device_id = self.extract_target_device(&bus_msg.payload).await?;

        let routing_table = self.routing_table.read().await;

        if let Some(target_source) = routing_table.get(&target_device_id) {
            log_info!("Routing message to device '{}' via {:?}", target_device_id, target_source.transport);

            // Convert to SSP format
            let ssp_msg = SspMessage::from_bus_message(
                bus_msg,
                target_source.transport.clone(),
                target_source.address.clone(),
            );

            // Send via appropriate transport
            self.send_via_transport(&ssp_msg, target_source).await?;
        } else {
            log_warn!("No routing info for device '{}', broadcasting to all transports", target_device_id);
        }

        Ok(())
    }

    /// Extract target device ID from message payload
    async fn extract_target_device(&self, payload: &str) -> Result<String> {
        // Try to parse as JSON and look for device_id or target fields
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(device_id) = json.get("device_id").and_then(|v| v.as_str()) {
                return Ok(device_id.to_string());
            }
            if let Some(target) = json.get("target").and_then(|v| v.as_str()) {
                return Ok(target.to_string());
            }
        }

        // Default: use payload topic or generic broadcast
        Ok("broadcast".to_string())
    }

    /// Send message via the specified transport
    async fn send_via_transport(&self, ssp_msg: &SspMessage, target: &SourceInfo) -> Result<()> {
        let wire_format = ssp_msg.to_wire();

        match target.transport {
            Transport::Usb | Transport::Ble => {
                self.send_serial(&target.address, &wire_format).await?;
            }
            Transport::Radio | Transport::Lora => {
                log_warn!("Radio/LoRa transport not yet implemented");
            }
            _ => {
                log_warn!("Unsupported transport: {:?}", target.transport);
            }
        }

        Ok(())
    }

    /// Send data over serial (USB or BLE Friend)
    async fn send_serial(&self, port_path: &str, data: &str) -> Result<()> {
        log_info!("Sending to serial port {}: {}", port_path, data.trim());

        // Open serial port
        let mut port = tokio_serial::new(port_path, 115200)
            .open_native_async()?;

        // Write data
        port.write_all(data.as_bytes()).await?;
        port.flush().await?;

        Ok(())
    }

    /// Start listening for inbound messages on USB serial ports
    async fn start_usb_listener(&self) -> Result<()> {
        // Check for available USB serial ports
        let ports = self.detect_usb_ports();

        if ports.is_empty() {
            log_info!("No USB serial ports detected, transport manager running in stub mode");
            return Ok(());
        }

        log_info!("Found {} serial port(s)", ports.len());
        for port in &ports {
            log_info!("  - {}", port);
        }

        for port_path in ports {
            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.listen_serial_port(port_path.clone()).await {
                    log_error!("Serial listener error on {}: {}", port_path, e);
                }
            });
        }

        Ok(())
    }

    /// Detect available USB serial ports
    fn detect_usb_ports(&self) -> Vec<String> {
        let mut ports = Vec::new();

        #[cfg(target_os = "linux")]
        {
            for i in 0..4 {
                let path = format!("/dev/ttyUSB{}", i);
                if std::path::Path::new(&path).exists() {
                    ports.push(path);
                }
            }
            for i in 0..4 {
                let path = format!("/dev/ttyACM{}", i);
                if std::path::Path::new(&path).exists() {
                    ports.push(path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS USB serial paths
            // Use ONLY cu.* devices (call-out) for bidirectional communication
            // Avoid tty.* to prevent "device busy" errors
            if let Ok(entries) = std::fs::read_dir("/dev") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Only use cu.* devices, not tty.*
                        if name.starts_with("cu.usbserial") ||
                            name.starts_with("cu.usbmodem") {
                            ports.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        ports
    }

    /// Listen for inbound messages on a serial port (USB or BLE Friend)
    async fn listen_serial_port(&self, port_path: String) -> Result<()> {
        log_info!("Starting serial listener on {}", port_path);

        // Open serial port
        let port = tokio_serial::new(&port_path, 115200)
            .open_native_async()?;

        let reader = BufReader::new(port);
        let mut lines = reader.lines();

        log_info!("Serial port {} opened successfully, listening for SSP messages...", port_path);

        while let Some(line) = lines.next_line().await? {
            // Skip empty lines and AT command responses
            let trimmed = line.trim();

            log_info!("🔵 RAW BYTES RECEIVED (len={}): '{}'", trimmed.len(), trimmed);

            if trimmed.is_empty() ||
                trimmed.starts_with("AT") ||
                trimmed == "OK" ||
                trimmed.starts_with("ERROR") {
                continue;
            }

            log_info!("Received raw line: {}", trimmed);

            // Try to parse as SSP message
            match SspMessage::parse(trimmed) {
                Ok(ssp_msg) => {
                    log_info!("✓ Parsed SSP message from {}: topic={}, type={:?}",
                             ssp_msg.source.id, ssp_msg.topic, ssp_msg.msg_type);

                    // Store routing info for this device
                    {
                        let mut routing_table = self.routing_table.write().await;
                        routing_table.insert(
                            ssp_msg.source.id.clone(),
                            ssp_msg.source.clone(),
                        );
                        log_info!("Updated routing table: {} devices known", routing_table.len());
                    }

                    // Convert to bus message and publish
                    let bus_msg = ssp_msg.to_bus_message();
                    if let Err(e) = self.message_bus.publish(bus_msg).await {
                        log_error!("Failed to publish inbound message: {}", e);
                    } else {
                        log_info!("✓ Published to message bus: topic={}", ssp_msg.topic);
                    }
                }
                Err(e) => {
                    log_warn!("Failed to parse SSP message: {} (line: {})", e, trimmed);
                }
            }
        }

        log_warn!("Serial port {} closed", port_path);
        Ok(())
    }
}

impl std::fmt::Debug for TransportManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportManager")
            .field("routing_table", &"<RwLock>")
            .field("message_bus", &"<MessageBus>")
            .field("outbound_topics", &"<RwLock>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ssp_roundtrip() {
        let (bus, _receiver) = MessageBus::new();
        let manager = TransportManager::new(bus.clone());

        // Create a test SSP message
        let ssp_msg = SspMessage {
            protocol: "ssp/1.0".to_string(),
            msg_type: MessageType::Telemetry,
            topic: "pressure_sensor".to_string(),
            timestamp: 1732377600,
            source: SourceInfo {
                id: "test_device".to_string(),
                transport: Transport::Usb,
                address: "/dev/ttyUSB0".to_string(),
            },
            payload: serde_json::json!({"pressure_psi": 85.5}),
            qos: None,
            retain: None,
            reply_to: None,
            in_reply_to: None,
        };

        // Convert to bus message
        let bus_msg = ssp_msg.to_bus_message();
        assert_eq!(bus_msg.topic, "pressure_sensor");
        assert_eq!(bus_msg.source, "test_device");

        // Convert back to SSP
        let ssp_back = SspMessage::from_bus_message(
            &bus_msg,
            ssp_msg.source.transport.clone(),
            ssp_msg.source.address.clone()
        );
        assert_eq!(ssp_back.topic, "pressure_sensor");
    }
}
