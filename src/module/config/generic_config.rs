use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::module::config::BaseModuleConfig;

/// Generic/system module (fallback)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: HashMap<String, serde_json::Value>,
}
