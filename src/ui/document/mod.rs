pub mod external_viewer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use color_eyre::Result;

use pdf::object::*;
use uuid::Uuid;
use std::fs;
use std::sync::Arc;

use external_viewer::ExternalViewer;

#[derive(Debug)]
pub struct DocumentManager {
    viewer: DocumentViewer,
    external_viewer: Option<Arc<ExternalViewer>>,
}

impl DocumentManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            viewer: DocumentViewer::new(),
            external_viewer: Some(Arc::new(ExternalViewer::new()?)),
        })
    }

    pub fn open_document(&mut self, file_path: String) {
        let (actual_path, _page_number) = if file_path.contains("#page=") {
            let parts: Vec<&str> = file_path.split("#page=").collect();
            let page = parts.get(1).and_then(|p| p.parse::<u32>().ok());
            (parts[0].to_string(), page)
        } else {
            (file_path.clone(), None)
        };

        let path = Path::new(&actual_path);
        let content = self.viewer.get_direct_view_content(path)
            .or_else(|| {
                self.viewer.view_document(path).ok()
            });

        if let Some(content) = content {
            if let Some(external_viewer) = &self.external_viewer {
                let viewer = external_viewer.clone();
                let path_clone = file_path.clone();

                tokio::spawn(async move {
                    if let Err(e) = viewer.show_document_external(&path_clone, &content).await {
                        eprintln!("Failed to launch external viewer: {}", e);
                    }
                });
            }
        }
    }
}

pub trait DocumentViewStrategy: Debug {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> Result<DocumentContent>;
    fn get_supported_extensions(&self) -> Vec<&'static str>;

    /// Returns true if this strategy's files can be viewed directly by a browser
    /// without parsing (e.g., PDFs, images, videos)
    fn supports_direct_viewing(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct DocumentContent {
    pub text: String,
    pub image_mappings: HashMap<String, String>,
    pub metadata: serde_json::Value,
}

impl DocumentContent {
    /// Create empty content for direct viewing (PDFs, media files)
    pub fn empty_for_direct_view(file_type: &str) -> Self {
        Self {
            text: String::new(),
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": file_type, "direct_view": true}),
        }
    }
}

#[derive(Debug)]
pub struct DocumentViewer {
    strategies: HashMap<String, Box<dyn DocumentViewStrategy>>,
}

impl DocumentViewer {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Box<dyn DocumentViewStrategy>> = HashMap::new();

        // Register strategies
        let pdf_strategy = PdfViewStrategy;
        for ext in pdf_strategy.get_supported_extensions() {
            strategies.insert(ext.to_string(), Box::new(PdfViewStrategy));
        }

        let text_strategy = TextViewStrategy;
        for ext in text_strategy.get_supported_extensions() {
            strategies.insert(ext.to_string(), Box::new(TextViewStrategy));
        }

        let media_strategy = MediaViewStrategy;
        for ext in media_strategy.get_supported_extensions() {
            strategies.insert(ext.to_string(), Box::new(MediaViewStrategy));
        }

        Self { strategies }
    }

    pub fn view_document(&self, file_path: &Path) -> Result<DocumentContent> {
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

// PDF Strategy - supports direct viewing
#[derive(Debug)]
pub struct PdfViewStrategy;

impl DocumentViewStrategy for PdfViewStrategy {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> Result<DocumentContent> {
        // This is only called if someone explicitly wants to parse the PDF
        // (e.g., for knowledge ingestion)
        let pdf_cache_dir = cache_dir.join(format!("pdf_{}", Uuid::new_v4()));
        fs::create_dir_all(&pdf_cache_dir)?;

        let text = pdf_extract::extract_text(file_path)?;
        Ok(DocumentContent {
            text,
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": "pdf"}),
        })
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }

    fn supports_direct_viewing(&self) -> bool {
        true // PDFs can be viewed directly in browsers
    }
}

// Text Strategy - needs parsing to create HTML
#[derive(Debug)]
pub struct TextViewStrategy;

impl DocumentViewStrategy for TextViewStrategy {
    fn parse_content(&self, file_path: &Path, _cache_dir: &Path) -> Result<DocumentContent> {
        let text = std::fs::read_to_string(file_path)?;
        Ok(DocumentContent {
            text,
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": "text"}),
        })
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "md", "log", "rtf"]
    }

    fn supports_direct_viewing(&self) -> bool {
        false // Text files need HTML conversion
    }
}

// Media Strategy - for images, video, audio (all support direct viewing)
#[derive(Debug)]
pub struct MediaViewStrategy;

impl DocumentViewStrategy for MediaViewStrategy {
    fn parse_content(&self, file_path: &Path, _cache_dir: &Path) -> Result<DocumentContent> {
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
