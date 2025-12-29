mod parse_content;

use std::path::Path;

use crate::ui::document::{
    content::DocumentContent,
    viewer::DocumentViewStrategy,
};

#[derive(Debug)]
pub struct MediaViewStrategy;

impl DocumentViewStrategy for MediaViewStrategy {
    fn parse_content(&self, file_path: &Path, _cache_dir: &Path) -> color_eyre::Result<DocumentContent> {
        self._parse_content(file_path, _cache_dir)
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec![
            // Images
            "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg",
            // Video
            "mp4", "webm", "ogg", "ogv", "avi", "mov", "mkv",
            // Audio
            "mp3", "wav", "oga", "flac", "m4a", "aac"
        ]
    }

    fn supports_direct_viewing(&self) -> bool {
        true // All media files can be viewed directly in HTML5
    }
}
