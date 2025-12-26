mod base;
mod name;
mod module_type;

use serde::{Deserialize, Serialize};

use crate::module::config::{
    GenericConfig,
    AlbumConfig,
    ComConfig,
    KnowledgeConfig,
    LlmConfig,
    MonitoringConfig,
    OverseerConfig,
    SideQuestConfig,
    ValveControlConfig
};

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
