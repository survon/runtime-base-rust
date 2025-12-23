use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::util::{
    database::Database,
    io::{
        bus::MessageBus,
        discovery::DiscoveryManager,
    }
};
use crate::modules::overseer::database::WastelandDatabase;

use super::{OverseerHandler, WastelandView};

impl OverseerHandler {
    pub fn new(
        wasteland_path: PathBuf,
        discovery_manager: Option<Arc<DiscoveryManager>>,
        database: Database,
        message_bus: MessageBus,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let archive_path = wasteland_path.join(".archive");

        let database_clone = database.clone();

        let mut handler = Self {
            current_view: WastelandView::Main,
            selected_index: 0,
            wasteland_path,
            archive_path,
            registry_url: "https://registry.survon.io/v1".to_string(),
            discovery_manager,
            database,
            message_bus: message_bus.clone(),
            message_tx,
            message_rx,
            pending_devices: Vec::new(),
            known_devices: Vec::new(),
            registry_modules: Vec::new(),
            installed_modules: Vec::new(),
            archived_modules: Vec::new(),
            status_message: None,
            is_scanning: false,
            scan_countdown: 0,
            config_editor: None,
        };

        // Start listening for device discovery events
        handler.start_device_listener();

        if let Ok(devices) = database_clone.get_all_known_devices() {
            handler.known_devices = devices;
        }

        // Do initial async refresh
        handler.refresh_installed_modules();

        if let Ok(archived) = handler.list_archived_modules() {
            handler.archived_modules = archived;
        }

        // Now do async refresh for things that need network/async ops
        handler.refresh_async_data_only();

        handler.status_message = Some("Ready - Press '[s]' to scan for devices".to_string());

        handler
    }
}
