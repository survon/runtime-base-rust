// src/util/bus/serial.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
}

impl SspMessage {
    pub fn parse(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
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
