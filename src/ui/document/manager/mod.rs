mod new;
mod open_document;

use std::sync::Arc;
use tokio::io::AsyncBufReadExt;

use crate::ui::document::{
    viewer::{
        DocumentViewer,
        external::ExternalViewer,
    },
};

#[derive(Debug)]
pub struct DocumentManager {
    viewer: DocumentViewer,
    external_viewer: Option<Arc<ExternalViewer>>,
}
