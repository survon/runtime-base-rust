mod parse_content;

use std::path::Path;

use crate::ui::document::{
    content::DocumentContent,
    viewer::DocumentViewStrategy,
};

#[derive(Debug)]
pub struct PdfViewStrategy;

impl DocumentViewStrategy for PdfViewStrategy {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        self._parse_content(file_path, cache_dir)
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }

    fn supports_direct_viewing(&self) -> bool {
        true // PDFs can be viewed directly in browsers
    }
}
