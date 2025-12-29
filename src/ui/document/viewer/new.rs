use std::collections::HashMap;

use crate::ui::document::{
    viewer::{
        DocumentViewer,
        DocumentViewStrategy,
        strategies::{
            MediaViewStrategy,
            PdfViewStrategy,
            TextViewStrategy,
        }
    }
};

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
}
