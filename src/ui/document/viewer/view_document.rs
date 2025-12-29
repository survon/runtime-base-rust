use std::path::{Path, PathBuf};

use crate::ui::document::{
    content::DocumentContent,
    viewer::DocumentViewer,
};

impl DocumentViewer {
    pub fn view_document(&self, file_path: &Path) -> color_eyre::Result<DocumentContent> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(strategy) = self.strategies.get(&extension) {
            let cache_dir = PathBuf::from("./.cache/knowledge");
            strategy.parse_content(file_path, &cache_dir)
        } else {
            Err(color_eyre::eyre::eyre!("Unsupported file type: {}", extension))
        }
    }
}
