use std::path::Path;

use crate::ui::document::viewer::DocumentViewer;

impl DocumentViewer {
    /// Check if a file can be viewed directly without parsing
    pub fn supports_direct_viewing(&self, file_path: &Path) -> bool {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        self.strategies.get(&extension)
            .map(|s| s.supports_direct_viewing())
            .unwrap_or(false)
    }
}
