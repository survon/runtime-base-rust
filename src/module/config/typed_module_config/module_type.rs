use super::TypedModuleConfig;

impl TypedModuleConfig {
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
