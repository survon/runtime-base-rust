use std::path::Path;

use crate::ui::document::{
    content::DocumentContent,
    viewer::DocumentViewer,
};

impl DocumentViewer {
    /// Get empty content for direct viewing files
    pub fn get_direct_view_content(&self, file_path: &Path) -> Option<DocumentContent> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if self.supports_direct_viewing(file_path) {
            Some(DocumentContent::empty_for_direct_view(&extension))
        } else {
            None
        }
    }
}
