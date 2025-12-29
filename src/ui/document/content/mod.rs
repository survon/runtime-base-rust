mod empty_for_direct_view;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DocumentContent {
    pub text: String,
    pub image_mappings: HashMap<String, String>,
    pub metadata: serde_json::Value,
}
