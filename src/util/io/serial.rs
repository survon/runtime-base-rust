// src/util/bus/serial.rs
use serde::{
    Deserialize,
    Serialize,
    de::Error as SerdeError
};
use std::collections::HashMap;

use crate::{log_debug,log_warn};
use crate::util::io::bus::{BusMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SspMessage {
    pub protocol: String,  // "ssp/1.0"
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub topic: String,
    pub timestamp: u64,
    pub source: SourceInfo,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qos: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Telemetry,
    Command,
    Response,
    Event,
}

impl MessageType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "telemetry" => MessageType::Telemetry,
            "command" => MessageType::Command,
            "response" => MessageType::Response,
            "event" => MessageType::Event,
            _ => MessageType::Telemetry,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub id: String,
    pub transport: Transport,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Ble,
    Radio,
    Usb,
    Lora,
    Zigbee,
    Internal,
    Unknown
}

impl Transport {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "usb" => Transport::Usb,
            "ble" => Transport::Ble,
            "lora" => Transport::Lora,
            "radio" => Transport::Radio,
            _ => Transport::Unknown,
        }
    }
}

impl SspMessage {
    /// Format: {"p":"ssp/1.0","t":"tel","i":"device_id","s":timestamp,"d":{"a":val1,"b":val2,...}}
    pub fn parse_flexible(json_str: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;

        log_debug!("Parsing compact SSP: {}", json_str);

        // Required fields (compact format)
        let protocol = value.get("p")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde_json::Error::custom("Missing 'p' (protocol)"))?;

        let msg_type_str = value.get("t")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde_json::Error::custom("Missing 't' (type)"))?;

        let device_id = value.get("i")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde_json::Error::custom("Missing 'i' (device id)"))?;

        let timestamp = value.get("s")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| serde_json::Error::custom("Missing 's' (timestamp)"))?;

        // Data payload
        let data = value.get("d")
            .and_then(|v| v.as_object())
            .ok_or_else(|| serde_json::Error::custom("Missing 'd' (data)"))?;

        // Expand message type shorthand
        let msg_type_expanded = match msg_type_str {
            "tel" => "telemetry",
            "cmd" => "command",
            "res" => "response",
            "evt" => "event",
            other => other,
        };

        // Keep data as-is - don't expand keys here
        // The module's config.yml bindings will map a,b,c to meaningful names
        let payload = serde_json::Value::Object(data.clone());

        // Source info (compact format uses device_id as both id and topic)
        let source = SourceInfo {
            id: device_id.to_string(),
            transport: Transport::Ble, // Inferred from compact format
            address: String::new(),
        };

        log_debug!(
            "✓ Parsed compact SSP - id:{}, type:{}, timestamp:{}, data keys:{:?}",
            device_id,
            msg_type_expanded,
            timestamp,
            data.keys().collect::<Vec<_>>()
        );

        Ok(SspMessage {
            protocol: protocol.to_string(),
            msg_type: MessageType::from_str(msg_type_expanded),
            topic: device_id.to_string(), // Use device_id as topic
            timestamp,
            source,
            payload,
            qos: None,
            retain: None,
            reply_to: None,
            in_reply_to: None,
        })
    }

    // Keep the old parse() for backward compatibility but log a warning
    pub fn parse(line: &str) -> Result<Self, serde_json::Error> {
        log_warn!("Legacy SSP format detected - use compact format instead");
        Self::parse_flexible(line)
    }

    pub fn to_wire(&self) -> String {
        format!("{}\n", serde_json::to_string(self).unwrap())
    }

    pub fn to_bus_message(&self) -> BusMessage {
        BusMessage::new(
            self.topic.clone(),
            serde_json::to_string(&self.payload).unwrap(),
            self.source.id.clone(),
        )
    }

    pub fn from_bus_message(bus_msg: &BusMessage, transport: Transport, address: String) -> Self {
        Self {
            protocol: "ssp/1.0".to_string(),
            msg_type: MessageType::Command, // or infer from topic
            topic: bus_msg.topic.clone(),
            timestamp: bus_msg.timestamp,
            source: SourceInfo {
                id: bus_msg.source.clone(),
                transport,
                address,
            },
            payload: serde_json::from_str(&bus_msg.payload)
                .unwrap_or_else(|_| serde_json::json!({"raw": bus_msg.payload})),
            qos: None,
            retain: None,
            reply_to: None,
            in_reply_to: None,
        }
    }
}
