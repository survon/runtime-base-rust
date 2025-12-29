use std::{
    collections::HashMap,
    fs,
    path::Path,
};
use uuid::Uuid;

use crate::ui::document::{
    content::DocumentContent,
    viewer::strategies::pdf::PdfViewStrategy
};

impl PdfViewStrategy {
    /// This is only called if someone explicitly wants to parse the PDF
    /// (e.g., for knowledge ingestion)
    pub(super) fn _parse_content(&self, file_path: &Path, cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        let pdf_cache_dir = cache_dir.join(format!("pdf_{}", Uuid::new_v4()));
        fs::create_dir_all(&pdf_cache_dir)?;

        let text = pdf_extract::extract_text(file_path)?;
        Ok(DocumentContent {
            text,
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": "pdf"}),
        })
    }
}
