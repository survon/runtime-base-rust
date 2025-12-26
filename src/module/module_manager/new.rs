use std::{
    collections::HashMap,
    path::PathBuf,

};

use crate::module::{
    ModuleManagerView,
    ModuleManager
};

impl ModuleManager {
    pub fn new(manifests_path: PathBuf, namespace: String) -> Self {
        Self {
            modules: Vec::new(),
            manifests_path,
            namespace,
            selected_module: 0,
            current_view: ModuleManagerView::ModuleListView,
            event_receivers: Vec::new(),
            handlers: HashMap::new(),
        }
    }
}
