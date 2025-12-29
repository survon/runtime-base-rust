use std::{
    collections::HashMap,
    path::Path,
};

use crate::ui::document::{
    content::DocumentContent,
    viewer::strategies::media::MediaViewStrategy,
};


impl MediaViewStrategy {
    pub(super) fn _parse_content(&self, file_path: &Path, _cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        // Media files don't need parsing, just return metadata
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        Ok(DocumentContent {
            text: String::new(),
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({
                "type": extension,
                "path": file_path.to_string_lossy()
            }),
        })
    }
}
