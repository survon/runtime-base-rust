// src/util/io/ble_scheduler.rs
//! BLE Command Scheduler - Queues commands and sends during advertised windows

use color_eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, sleep_until};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use btleplug::api::{Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

use crate::log_info;
use crate::log_warn;
use crate::log_error;
use crate::util::io::bus::{MessageBus, BusMessage};

/// Command to be sent to a device
#[derive(Debug, Clone)]
pub struct QueuedCommand {
    pub device_id: String,
    pub command: serde_json::Value,
    pub priority: CommandPriority,
    pub queued_at: Instant,
    pub max_age: Option<Duration>,  // Optional expiration
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,  // Send immediately, don't wait for window
}

/// Device schedule information extracted from telemetry
#[derive(Debug, Clone)]
pub struct DeviceSchedule {
    pub device_id: String,
    pub current_mode: DeviceMode,
    pub cmd_window_opens_at: Option<Instant>,  // When next CMD window opens
    pub cmd_window_duration: Duration,         // How long it stays open
    pub last_updated: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceMode {
    Data,
    Cmd,
}

impl DeviceSchedule {
    /// Check if device is currently in CMD window
    pub fn is_in_cmd_window(&self) -> bool {
        self.current_mode == DeviceMode::Cmd
    }

    /// Check if CMD window is imminent (within next 5 seconds)
    pub fn is_cmd_window_imminent(&self) -> bool {
        if let Some(opens_at) = self.cmd_window_opens_at {
            let now = Instant::now();
            opens_at > now && opens_at.duration_since(now) < Duration::from_secs(5)
        } else {
            false
        }
    }

    /// Get time until CMD window opens (None if in window or unknown)
    pub fn time_until_cmd_window(&self) -> Option<Duration> {
        if self.current_mode == DeviceMode::Cmd {
            return None;  // Already in window
        }

        if let Some(opens_at) = self.cmd_window_opens_at {
            let now = Instant::now();
            if opens_at > now {
                Some(opens_at.duration_since(now))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Command scheduler manages queues and timing for all devices
#[derive(Debug, Clone)]
pub struct BleCommandScheduler {
    /// Command queues per device
    command_queues: Arc<RwLock<HashMap<String, Vec<QueuedCommand>>>>,

    /// Schedule information per device
    device_schedules: Arc<RwLock<HashMap<String, DeviceSchedule>>>,

    /// Active peripherals (needed for sending)
    peripherals: Arc<RwLock<HashMap<String, Peripheral>>>,

    /// TX characteristic UUID
    tx_char_uuid: Uuid,

    /// Message bus for publishing events
    message_bus: Option<MessageBus>,
}

impl BleCommandScheduler {
    pub fn new() -> Self {
        Self {
            command_queues: Arc::new(RwLock::new(HashMap::new())),
            device_schedules: Arc::new(RwLock::new(HashMap::new())),
            peripherals: Arc::new(RwLock::new(HashMap::new())),
            tx_char_uuid: Uuid::parse_str("6e400002-b5a3-f393-e0a9-e50e24dcca9e").unwrap(),
            message_bus: None,
        }
    }

    /// Builder method to add message bus for event publishing
    pub fn with_message_bus(mut self, bus: MessageBus) -> Self {
        self.message_bus = Some(bus);
        self
    }

    /// Register a peripheral for command sending
    pub async fn register_peripheral(&self, device_id: String, peripheral: Peripheral) {
        self.peripherals.write().await.insert(device_id, peripheral);
    }

    /// Queue a command for a device
    pub async fn queue_command(&self, mut command: QueuedCommand) -> Result<()> {
        command.queued_at = Instant::now();

        let device_id = command.device_id.clone();
        let priority = command.priority;

        log_info!(
            "📥 Queuing {} priority command for device {}",
            match priority {
                CommandPriority::Critical => "CRITICAL",
                CommandPriority::High => "HIGH",
                CommandPriority::Normal => "NORMAL",
                CommandPriority::Low => "LOW",
            },
            device_id
        );

        // Critical commands bypass the queue and send immediately
        if priority == CommandPriority::Critical {
            log_warn!("⚡ CRITICAL command - sending immediately (ignoring window)");

            self.publish_event(
                "command_sent_critical",
                &device_id,
                serde_json::json!({
                    "priority": "CRITICAL",
                    "action": command.command.get("payload")
                        .and_then(|p| p.get("action"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("unknown")
                })
            ).await;

            return self.send_command_now(&device_id, &command.command).await;
        }

        // Add to queue
        let mut queues = self.command_queues.write().await;
        let queue = queues.entry(device_id.clone()).or_insert_with(Vec::new);
        queue.push(command.clone());

        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        let queue_size = queue.len();
        log_info!("📋 Device {} now has {} queued commands", device_id, queue_size);

        // Publish queue events
        self.publish_event(
            "command_queued",
            &device_id,
            serde_json::json!({
                "priority": format!("{:?}", priority),
                "action": command.command.get("payload")
                    .and_then(|p| p.get("action"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown"),
                "queue_size": queue_size
            })
        ).await;

        Ok(())
    }

    /// Update device schedule from telemetry metadata
    /// Expected metadata format: {"mode": "data", "cmd_in": 285, "cmd_dur": 10}
    pub async fn update_schedule_from_telemetry(
        &self,
        device_id: String,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        let mode = match metadata.get("mode").and_then(|v| v.as_str()) {
            Some("cmd") => DeviceMode::Cmd,
            Some("data") => DeviceMode::Data,
            _ => return Ok(()),  // No mode info, skip
        };

        let cmd_in_secs = metadata.get("cmd_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let cmd_dur_secs = metadata.get("cmd_dur")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let cmd_window_opens_at = if cmd_in_secs > 0 {
            Some(Instant::now() + Duration::from_secs(cmd_in_secs))
        } else {
            None  // Already in CMD window or unknown
        };

        let schedule = DeviceSchedule {
            device_id: device_id.clone(),
            current_mode: mode,
            cmd_window_opens_at,
            cmd_window_duration: Duration::from_secs(cmd_dur_secs),
            last_updated: Instant::now(),
        };

        // Check if we should send commands NOW
        let should_send_now = schedule.is_in_cmd_window();
        let window_imminent = schedule.is_cmd_window_imminent();

        self.device_schedules.write().await.insert(device_id.clone(), schedule);

        if should_send_now {
            log_info!("🟢 Device {} is in CMD window - sending queued commands", device_id);

            self.publish_event(
                "cmd_window_open",
                &device_id,
                serde_json::json!({ "duration": cmd_dur_secs })
            ).await;

            self.send_queued_commands(&device_id).await?;
        } else if window_imminent {
            log_info!("🟡 Device {} CMD window opens in <5s - preparing", device_id);

            self.publish_event(
                "cmd_window_imminent",
                &device_id,
                serde_json::json!({ "seconds": cmd_in_secs })
            ).await;
        } else if let Some(time_until) = self.device_schedules.read().await.get(&device_id)
            .and_then(|s| s.time_until_cmd_window()) {
            log_info!(
                "⏰ Device {} CMD window in {}s",
                device_id,
                time_until.as_secs()
            );

            self.publish_event(
                "cmd_window_scheduled",
                &device_id,
                serde_json::json!({ "seconds": time_until.as_secs() })
            ).await;
        }

        Ok(())
    }

    /// Send all queued commands for a device
    async fn send_queued_commands(&self, device_id: &str) -> Result<()> {
        let mut queues = self.command_queues.write().await;
        let queue = match queues.get_mut(device_id) {
            Some(q) if !q.is_empty() => q,
            _ => {
                log_info!("📭 No queued commands for {}", device_id);
                return Ok(());
            }
        };

        let original_count = queue.len();
        log_info!("📤 Sending {} queued commands to {}", original_count, device_id);

        // Publish batch start
        self.publish_event(
            "batch_start",
            device_id,
            serde_json::json!({ "count": original_count })
        ).await;

        // Remove expired commands
        let now = Instant::now();
        queue.retain(|cmd| {
            if let Some(max_age) = cmd.max_age {
                if now.duration_since(cmd.queued_at) > max_age {
                    log_warn!("⏳ Dropping expired command for {}", device_id);
                    return false;
                }
            }
            true
        });

        let expired_count = original_count - queue.len();
        if expired_count > 0 {
            self.publish_event(
                "commands_expired",
                device_id,
                serde_json::json!({ "count": expired_count })
            ).await;
        }

        // Send commands in priority order
        let commands = queue.drain(..).collect::<Vec<_>>();
        drop(queues);  // Release lock before sending

        for cmd in commands {
            log_info!("📨 Sending command to {}", device_id);

            // Small delay between commands to avoid overwhelming device
            tokio::time::sleep(Duration::from_millis(100)).await;

            match self.send_command_now(device_id, &cmd.command).await {
                Ok(_) => {
                    self.publish_event(
                        "command_sent",
                        device_id,
                        serde_json::json!({
                            "action": cmd.command.get("payload")
                                .and_then(|p| p.get("action"))
                                .and_then(|a| a.as_str())
                                .unwrap_or("unknown")
                        })
                    ).await;
                }
                Err(e) => {
                    log_error!("❌ Failed to send command: {}", e);
                    self.publish_event(
                        "error",
                        device_id,
                        serde_json::json!({ "error": e.to_string() })
                    ).await;
                }
            }
        }

        // Publish batch complete
        self.publish_event(
            "batch_complete",
            device_id,
            serde_json::json!({ "count": original_count - expired_count })
        ).await;

        Ok(())
    }

    /// Send a single command immediately (used for critical commands)
    async fn send_command_now(&self, device_id: &str, command: &serde_json::Value) -> Result<()> {
        let peripherals = self.peripherals.read().await;
        let peripheral = peripherals.get(device_id)
            .ok_or_else(|| color_eyre::eyre::eyre!("Device {} not registered", device_id))?;

        // Find TX characteristic
        let chars = peripheral.characteristics();
        let tx_char = chars.iter()
            .find(|c| c.uuid == self.tx_char_uuid)
            .ok_or_else(|| color_eyre::eyre::eyre!("TX characteristic not found"))?;

        // Serialize and send
        let json_str = serde_json::to_string(command)?;
        let bytes = json_str.as_bytes();

        log_info!("📡 TX: {}", json_str);

        peripheral.write(tx_char, bytes, WriteType::WithoutResponse).await?;

        Ok(())
    }

    /// Get queue status for a device
    pub async fn get_queue_status(&self, device_id: &str) -> Option<QueueStatus> {
        let queues = self.command_queues.read().await;
        let schedules = self.device_schedules.read().await;

        let queue = queues.get(device_id)?;
        let schedule = schedules.get(device_id)?;

        Some(QueueStatus {
            device_id: device_id.to_string(),
            queued_commands: queue.len(),
            current_mode: schedule.current_mode,
            time_until_cmd_window: schedule.time_until_cmd_window(),
        })
    }

    /// Prune stale schedules (not updated in 5 minutes)
    pub async fn prune_stale_schedules(&self) {
        let mut schedules = self.device_schedules.write().await;
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(300);

        schedules.retain(|device_id, schedule| {
            let is_stale = now.duration_since(schedule.last_updated) > stale_threshold;
            if is_stale {
                log_warn!("🗑️ Removing stale schedule for device {}", device_id);
            }
            !is_stale
        });
    }

    /// Helper to publish events to message bus
    async fn publish_event(&self, event_type: &str, device_id: &str, details: serde_json::Value) {
        if let Some(bus) = &self.message_bus {
            let mut payload = serde_json::json!({
                "event": event_type,
                "device_id": device_id,
            });

            // Merge in the details
            if let Some(obj) = payload.as_object_mut() {
                if let Some(details_obj) = details.as_object() {
                    for (k, v) in details_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }

            let msg = BusMessage::new(
                "scheduler_event".to_string(),
                payload.to_string(),
                "ble_scheduler".to_string(),
            );

            let _ = bus.publish(msg).await;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueStatus {
    pub device_id: String,
    pub queued_commands: usize,
    pub current_mode: DeviceMode,
    pub time_until_cmd_window: Option<Duration>,
}

// ============================================================================
// HELPER FUNCTIONS FOR DISCOVERY MANAGER
// ============================================================================

/// Parse telemetry and extract schedule metadata
pub fn extract_schedule_metadata(telemetry: &serde_json::Value) -> Option<serde_json::Value> {
    telemetry.get("m").cloned()
}

/// Create a control command in SSP format
pub fn create_control_command(device_id: &str, action: &str, payload: Option<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "protocol": "ssp/1.0",
        "type": "control",
        "topic": device_id,
        "timestamp": chrono::Utc::now().timestamp() as u64,
        "payload": {
            "action": action,
            "data": payload
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_parsing() {
        let scheduler = BleCommandScheduler::new();

        let metadata = serde_json::json!({
            "mode": "data",
            "cmd_in": 285,
            "cmd_dur": 10
        });

        scheduler.update_schedule_from_telemetry(
            "test_device".to_string(),
            &metadata
        ).await.unwrap();

        let schedules = scheduler.device_schedules.read().await;
        let schedule = schedules.get("test_device").unwrap();

        assert_eq!(schedule.current_mode, DeviceMode::Data);
        assert!(schedule.time_until_cmd_window().is_some());
    }

    #[tokio::test]
    async fn test_command_queueing() {
        let scheduler = BleCommandScheduler::new();

        let cmd = QueuedCommand {
            device_id: "test_device".to_string(),
            command: serde_json::json!({"action": "ping"}),
            priority: CommandPriority::Normal,
            queued_at: Instant::now(),
            max_age: None,
        };

        scheduler.queue_command(cmd).await.unwrap();

        let queues = scheduler.command_queues.read().await;
        let queue = queues.get("test_device").unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn test_priority_sorting() {
        let scheduler = BleCommandScheduler::new();

        // Queue low priority first
        scheduler.queue_command(QueuedCommand {
            device_id: "test".to_string(),
            command: serde_json::json!({"action": "low"}),
            priority: CommandPriority::Low,
            queued_at: Instant::now(),
            max_age: None,
        }).await.unwrap();

        // Then high priority
        scheduler.queue_command(QueuedCommand {
            device_id: "test".to_string(),
            command: serde_json::json!({"action": "high"}),
            priority: CommandPriority::High,
            queued_at: Instant::now(),
            max_age: None,
        }).await.unwrap();

        let queues = scheduler.command_queues.read().await;
        let queue = queues.get("test").unwrap();

        // High priority should be first
        assert_eq!(queue[0].priority, CommandPriority::High);
        assert_eq!(queue[1].priority, CommandPriority::Low);
    }
}
