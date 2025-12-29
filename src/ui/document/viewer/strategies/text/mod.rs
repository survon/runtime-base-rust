mod parse_content;

use std::path::Path;

use crate::ui::document::{
    content::DocumentContent,
    viewer::DocumentViewStrategy,
};

#[derive(Debug)]
pub struct TextViewStrategy;

impl DocumentViewStrategy for TextViewStrategy {
    fn parse_content(&self, file_path: &Path, _cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        self._parse_content(file_path, _cache_dir)
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "md", "log", "rtf"]
    }

    fn supports_direct_viewing(&self) -> bool {
        false // Text files need HTML conversion
    }
}
