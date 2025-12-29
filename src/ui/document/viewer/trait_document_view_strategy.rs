use std::fmt::Debug;
use std::path::Path;

use crate::ui::document::content::DocumentContent;

pub trait DocumentViewStrategy: Debug {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> color_eyre::Result<DocumentContent>;
    fn get_supported_extensions(&self) -> Vec<&'static str>;

    /// Returns true if this strategy's files can be viewed directly by a browser
    /// without parsing (e.g., PDFs, images, videos)
    fn supports_direct_viewing(&self) -> bool {
        false
    }
}
