mod trait_document_view_strategy;
mod new;
mod strategies;
mod view_document;
mod supports_direct_viewing;
mod get_direct_view_content;
pub mod external;

use std::collections::HashMap;

pub use trait_document_view_strategy::DocumentViewStrategy;

#[derive(Debug)]
pub struct DocumentViewer {
    strategies: HashMap<String, Box<dyn DocumentViewStrategy>>,
}
