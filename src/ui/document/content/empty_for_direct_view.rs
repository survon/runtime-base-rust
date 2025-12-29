use std::collections::HashMap;

use super::DocumentContent;

impl DocumentContent {
    /// Create empty content for direct viewing (PDFs, media files)
    pub fn empty_for_direct_view(file_type: &str) -> Self {
        Self {
            text: String::new(),
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": file_type, "direct_view": true}),
        }
    }
}
