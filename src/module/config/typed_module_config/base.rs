use crate::module::config::{
    BaseModuleConfig,
    TypedModuleConfig
};

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
}
