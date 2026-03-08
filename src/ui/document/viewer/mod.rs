pub mod external;
mod get_direct_view_content;
mod new;
mod strategies;
mod supports_direct_viewing;
mod trait_document_view_strategy;
mod view_document;

use std::collections::HashMap;

pub use trait_document_view_strategy::DocumentViewStrategy;

#[derive(Debug)]
pub struct DocumentViewer {
    strategies: HashMap<String, Box<dyn DocumentViewStrategy>>,
}
