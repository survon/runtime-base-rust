// src/modules/config_schema.rs
// Type-safe configuration schemas for different module types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base configuration that all modules must have
///
/// *Note: `module_type` is NOT included because it's consumed by the enum's tag
/// since the enum variant already tells us the type*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModuleConfig {
    pub name: String,
    pub bus_topic: String,
    pub template: String,

    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}

/// Monitoring module (gauges, charts, status badges)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: MonitoringBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringBindings {
    // SSP compact data keys
    pub a: f64,
    pub b: f64,
    pub c: f64,

    // Device metadata
    pub device_id: String,
    pub device_type: String,
    pub firmware_version: String,

    // Display configuration
    pub display_name: String,
    pub unit_of_measure_label: String,

    // Thresholds (optional, depends on template)
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub warn_threshold: Option<f64>,
    #[serde(default)]
    pub danger_threshold: Option<f64>,

    // Chart-specific (optional)
    #[serde(default)]
    pub chart_type: Option<String>, // "line", "bar", "sparkline"

    // Connection tracking
    #[serde(default)]
    pub is_connected: Option<bool>,
    #[serde(default)]
    pub seconds_since_update: Option<i64>,
    #[serde(default)]
    pub status_suffix: Option<String>,

    // Internal state
    #[serde(default)]
    pub _chart_history: Option<Vec<f64>>,
}

/// Valve control module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveControlConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: ValveControlBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveControlBindings {
    // SSP compact data
    pub a: i32, // valve_open: 0=closed, 1=open
    pub b: i32, // position: 0-100%
    pub c: i32, // message_count

    // Device metadata
    pub device_id: String,
    pub device_type: String,
    pub firmware_version: String,

    // UI display
    pub state: bool, // Derived from "a"
    pub label: String,
    pub toggle_on_label: String,
    pub toggle_off_label: String,
    pub description: String,
    pub is_blinkable: bool,
}

/// LLM module (chat interfaces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub model: String, // "search", "summarizer", "council"
    pub bindings: LlmBindings,

    #[serde(default)]
    pub service_discovery: Option<ServiceDiscoveryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBindings {
    pub model_info: String,
    pub chat_history: Vec<serde_json::Value>,
    pub chat_input: String,
    pub scroll_offset: i32,

    // Council-specific
    #[serde(default)]
    pub available_advisors: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub active_advisor: Option<serde_json::Value>,
    #[serde(default)]
    pub current_link_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub method: String,
    pub scan_pattern: String,
    pub metadata_path: String,
    pub api_port: u16,
    pub web_port: u16,
    pub scan_interval_seconds: u64,
    pub required_contract: String,
}

/// Side Quest module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuestConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: SideQuestBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuestBindings {
    pub current_view: String, // "QuestList", "CreateQuest", "QuestDetail"
    pub selected_index: i32,
    pub quests: Vec<String>,
    pub quest_count: i32,

    // Create form state
    pub create_step: String,
    pub form_title: String,
    pub form_description: String,
    pub form_topic: String,
    pub form_urgency: String,

    // Available options
    pub available_topics: Vec<String>,
    pub urgency_options: Vec<String>,

    // Detail view
    pub selected_quest_title: String,
    pub selected_quest_description: String,
    pub selected_quest_topic: String,
    pub selected_quest_urgency: String,
    pub selected_quest_trigger: String,

    #[serde(default)]
    pub status_message: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}

/// Wasteland Manager module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: OverseerBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerBindings {
    pub current_view: String,
    pub selected_index: i32,
    pub pending_devices: Vec<String>,
    pub known_devices: Vec<String>,
    pub module_list: Vec<String>,
    pub installed_modules: Vec<String>,
    pub archived_modules: Vec<String>,

    #[serde(default)]
    pub status_message: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}

/// Album module (audio collections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: AlbumBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumBindings {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub genre: String,
    pub credits: String,
    pub tracklist: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub file: String,
    pub duration_seconds: i32,
    pub artist: String,
}

/// Knowledge module (document collections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: KnowledgeBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBindings {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Communication module (toggle switches, activity logs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: ComBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComBindings {
    // For toggle_switch template
    #[serde(default)]
    pub state: Option<bool>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub toggle_on_label: Option<String>,
    #[serde(default)]
    pub toggle_off_label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ports: Option<Vec<String>>,
    #[serde(default)]
    pub messages: Option<Vec<String>>,

    // For activity_card template
    #[serde(default)]
    pub activity_log: Option<Vec<String>>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Generic/system module (fallback)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: HashMap<String, serde_json::Value>,
}

/// Enum wrapping all possible module configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "module_type")]
pub enum TypedModuleConfig {
    #[serde(rename = "monitoring")]
    Monitoring(MonitoringConfig),

    #[serde(rename = "valve_control")]
    ValveControl(ValveControlConfig),

    #[serde(rename = "llm")]
    Llm(LlmConfig),

    #[serde(rename = "side_quest")]
    SideQuest(SideQuestConfig),

    #[serde(rename = "overseer")]
    Overseer(OverseerConfig),

    #[serde(rename = "album")]
    Album(AlbumConfig),

    #[serde(rename = "knowledge")]
    Knowledge(KnowledgeConfig),

    #[serde(rename = "com")]
    Com(ComConfig),

    #[serde(rename = "system")]
    System(GenericConfig),

    #[serde(other)]
    Unknown,
}

impl TypedModuleConfig {
    /// Get the base config regardless of type
    pub fn base(&self) -> Option<&BaseModuleConfig> {
        match self {
            Self::Monitoring(c) => Some(&c.base),
            Self::ValveControl(c) => Some(&c.base),
            Self::Llm(c) => Some(&c.base),
            Self::SideQuest(c) => Some(&c.base),
            Self::Overseer(c) => Some(&c.base),
            Self::Album(c) => Some(&c.base),
            Self::Knowledge(c) => Some(&c.base),
            Self::Com(c) => Some(&c.base),
            Self::System(c) => Some(&c.base),
            Self::Unknown => None,
        }
    }

    /// Get module name
    pub fn name(&self) -> Option<&str> {
        self.base().map(|b| b.name.as_str())
    }

    /// Get module type string
    pub fn module_type(&self) -> &str {
        match self {
            Self::Monitoring(_) => "monitoring",
            Self::ValveControl(_) => "valve_control",
            Self::Llm(_) => "llm",
            Self::SideQuest(_) => "side_quest",
            Self::Overseer(_) => "overseer",
            Self::Album(_) => "album",
            Self::Knowledge(_) => "knowledge",
            Self::Com(_) => "com",
            Self::System(_) => "system",
            Self::Unknown => "unknown",
        }
    }
}

/// Supported template list
pub fn get_supported_templates() -> Vec<&'static str> {
    vec![
        "gauge_card",
        "chart_card",
        "status_badge_card",
        "toggle_switch",
        "activity_card",
        "llm_card",
        "side_quest_card",
        "overseer_card",
        "",  // Empty template for hidden modules
    ]
}

/// Supported module types
pub fn get_supported_module_types() -> Vec<&'static str> {
    vec![
        "monitoring",
        "valve_control",
        "llm",
        "side_quest",
        "overseer",
        "album",
        "knowledge",
        "com",
        "system",
    ]
}
