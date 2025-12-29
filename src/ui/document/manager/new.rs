use std::sync::Arc;

use crate::ui::document::{
    manager::DocumentManager,
    viewer::{
        DocumentViewer,
        external::ExternalViewer,
    },
};

impl DocumentManager {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            viewer: DocumentViewer::new(),
            external_viewer: Some(Arc::new(ExternalViewer::new()?)),
        })
    }
}
