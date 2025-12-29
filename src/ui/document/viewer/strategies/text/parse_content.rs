use std::{
    collections::HashMap,
    path::Path,
};
use crate::ui::document::{
    content::DocumentContent,
    viewer::strategies::text::TextViewStrategy,
};

impl TextViewStrategy {
    pub(super) fn _parse_content(&self, file_path: &Path, _cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        let text = std::fs::read_to_string(file_path)?;
        Ok(DocumentContent {
            text,
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": "text"}),
        })
    }
}
