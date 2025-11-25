// src/modules/wasteland_manager/handler.rs
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::io::event::AppEvent;
use crate::util::io::discovery::DiscoveryManager;
use crate::util::wasteland_manager::WastelandManager;

#[derive(Debug, Clone)]
enum HandlerMessage {
    DevicesRefreshed(Vec<(String, String, i16)>),
    TrustedDevicesRefreshed(Vec<(String, String)>),
    RegistryRefreshed(Vec<crate::util::wasteland_manager::RegistryModule>),
    DeviceTrusted(String), // mac address
    ModuleInstalled(String),
    OperationInProgress(String), // status message
}

#[derive(Debug, Clone, PartialEq)]
enum WastelandView {
    Main,              // Show menu options
    TrustDevices,      // List discovered devices to trust
    InstallRegistry,   // Browse registry modules
    ManageModules,     // Edit/archive existing modules
    ArchivedModules,   // View archived modules
}

#[derive(Debug)]
pub struct WastelandManagerHandler {
    current_view: WastelandView,
    selected_index: usize,
    wasteland_manager: WastelandManager,
    discovery_manager: Option<Arc<DiscoveryManager>>,
    // Channel for async updates
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,
    // Cached data
    discovered_devices: Vec<(String, String, i16)>, // (mac, name, rssi)
    trusted_devices: Vec<(String, String)>,          // (mac, name)
    registry_modules: Vec<crate::util::wasteland_manager::RegistryModule>,
    installed_modules: Vec<String>,
    archived_modules: Vec<String>,
    // Status message
    status_message: Option<String>,
}

impl WastelandManagerHandler {
    pub fn new(
        wasteland_path: std::path::PathBuf,
        discovery_manager: Option<Arc<DiscoveryManager>>,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            current_view: WastelandView::Main,
            selected_index: 0,
            wasteland_manager: WastelandManager::new(wasteland_path.clone()),
            discovery_manager,
            message_tx,
            message_rx,
            discovered_devices: Vec::new(),
            trusted_devices: Vec::new(),
            registry_modules: Vec::new(),
            installed_modules: Vec::new(),
            archived_modules: Vec::new(),
            status_message: None,
        };

        // Do initial async refresh
        handler.refresh_data_async();
        handler
    }

    fn process_messages(&mut self) {
        // Non-blocking: process all queued messages
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::DevicesRefreshed(devices) => {
                    self.discovered_devices = devices;
                    self.status_message = None;
                }
                HandlerMessage::TrustedDevicesRefreshed(trusted) => {
                    self.trusted_devices = trusted;
                }
                HandlerMessage::RegistryRefreshed(modules) => {
                    self.registry_modules = modules;
                    self.status_message = None;
                }
                HandlerMessage::DeviceTrusted(mac) => {
                    self.discovered_devices.retain(|(m, _, _)| m != &mac);
                    if self.selected_index > 0 && self.selected_index >= self.discovered_devices.len() {
                        self.selected_index -= 1;
                    }
                    self.status_message = Some(format!("✓ Device {} trusted!", mac));
                }
                HandlerMessage::ModuleInstalled(name) => {
                    self.refresh_installed_modules();
                    self.status_message = Some(format!("✓ Module {} installed!", name));
                }
                HandlerMessage::OperationInProgress(msg) => {
                    self.status_message = Some(msg);
                }
            }
        }
    }

    fn refresh_data_async(&mut self) {
        let tx = self.message_tx.clone();

        // Refresh discovered devices
        if let Some(discovery) = &self.discovery_manager {
            let discovery_clone = discovery.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let devices = discovery_clone.get_discovered_devices().await;
                let _ = tx_clone.send(HandlerMessage::DevicesRefreshed(devices));
            });

            let discovery_clone = discovery.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Ok(trusted) = discovery_clone.get_trusted_devices().await {
                    let _ = tx_clone.send(HandlerMessage::TrustedDevicesRefreshed(trusted));
                }
            });
        }

        // Refresh archived modules (sync - fast enough)
        if let Ok(archived) = self.wasteland_manager.list_archived_modules() {
            self.archived_modules = archived;
        }

        // Refresh installed modules (sync - fast enough)
        self.refresh_installed_modules();

        // Refresh registry
        let manager = self.wasteland_manager.clone();
        tokio::spawn(async move {
            if let Ok(modules) = manager.list_registry_modules().await {
                let _ = tx.send(HandlerMessage::RegistryRefreshed(modules));
            }
        });
    }

    fn get_main_menu_items(&self) -> Vec<String> {
        vec![
            format!("📡 Trust BLE Devices ({})", self.discovered_devices.len()),
            format!("📦 Install from Registry ({})", self.registry_modules.len()),
            format!("⚙️  Manage Installed Modules ({})", self.installed_modules.len()),
            format!("📚 View Archived Modules ({})", self.archived_modules.len()),
            "← Back".to_string(),
        ]
    }

    fn handle_main_menu_select(&mut self) {
        match self.selected_index {
            0 => {
                self.current_view = WastelandView::TrustDevices;
                self.selected_index = 0;
                self.refresh_discovered_devices();
            }
            1 => {
                self.current_view = WastelandView::InstallRegistry;
                self.selected_index = 0;
            }
            2 => {
                self.current_view = WastelandView::ManageModules;
                self.selected_index = 0;
                self.refresh_installed_modules();
            }
            3 => {
                self.current_view = WastelandView::ArchivedModules;
                self.selected_index = 0;
                self.refresh_data_async();
            }
            4 => {
                // Back - will be handled by returning AppEvent::Back
            }
            _ => {}
        }
    }

    fn refresh_discovered_devices(&mut self) {
        let tx = self.message_tx.clone();
        self.status_message = Some("Refreshing devices...".to_string());

        if let Some(discovery) = &self.discovery_manager {
            let discovery_clone = discovery.clone();
            tokio::spawn(async move {
                let devices = discovery_clone.get_discovered_devices().await;
                let _ = tx.send(HandlerMessage::DevicesRefreshed(devices));
            });
        }
    }

    fn refresh_installed_modules(&mut self) {
        // Actually scan the wasteland directory
        use std::fs;

        let wasteland_path = std::path::PathBuf::from("./modules/wasteland/");
        self.installed_modules.clear();

        if let Ok(entries) = fs::read_dir(&wasteland_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            // Skip hidden directories
                            if !name_str.starts_with('.') {
                                self.installed_modules.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_trust_device(&mut self) {
        if self.selected_index < self.discovered_devices.len() {
            let (mac, name, _rssi) = &self.discovered_devices[self.selected_index];
            let mac_clone = mac.clone();
            let name_clone = name.clone();
            let tx = self.message_tx.clone();

            self.status_message = Some(format!("⏳ Trusting {}...", name_clone));

            if let Some(discovery) = &self.discovery_manager {
                let discovery_clone = discovery.clone();
                tokio::spawn(async move {
                    let _ = tx.send(HandlerMessage::OperationInProgress(
                        format!("Connecting to {}...", name_clone)
                    ));

                    match discovery_clone.trust_device(mac_clone.clone()).await {
                        Ok(_) => {
                            let _ = tx.send(HandlerMessage::DeviceTrusted(mac_clone));
                        }
                        Err(e) => {
                            let _ = tx.send(HandlerMessage::OperationInProgress(
                                format!("❌ Failed to trust device: {}", e)
                            ));
                        }
                    }
                });
            }
        }
    }

    fn handle_install_module(&mut self) {
        if self.selected_index < self.registry_modules.len() {
            let module = &self.registry_modules[self.selected_index];
            let module_id = module.id.clone();
            let module_name = module.name.clone();
            let manager = self.wasteland_manager.clone();
            let tx = self.message_tx.clone();

            self.status_message = Some(format!("⏳ Installing {}...", module_name));

            tokio::spawn(async move {
                match manager.install_module(
                    crate::util::wasteland_manager::InstallSource::Registry(module_id.clone()),
                    None
                ).await {
                    Ok(name) => {
                        let _ = tx.send(HandlerMessage::ModuleInstalled(name));
                    }
                    Err(e) => {
                        let _ = tx.send(HandlerMessage::OperationInProgress(
                            format!("❌ Failed to install: {}", e)
                        ));
                    }
                }
            });
        }
    }

    fn handle_archive_module(&mut self) {
        if self.selected_index < self.installed_modules.len() {
            let module_name = &self.installed_modules[self.selected_index];
            let _ = self.wasteland_manager.archive_module(module_name);

            self.installed_modules.remove(self.selected_index);
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }

    fn handle_restore_module(&mut self) {
        if self.selected_index < self.archived_modules.len() {
            let archive_name = &self.archived_modules[self.selected_index];
            let _ = self.wasteland_manager.restore_module(archive_name, None);

            self.archived_modules.remove(self.selected_index);
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }
}

impl ModuleHandler for WastelandManagerHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match self.current_view {
            WastelandView::Main => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.get_main_menu_items().len() - 1;
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        if self.selected_index == 4 {
                            // Back option
                            Some(AppEvent::Back)
                        } else {
                            self.handle_main_menu_select();
                            None
                        }
                    }
                    KeyCode::Esc => {
                        Some(AppEvent::Back)
                    }
                    KeyCode::Char('r') => {
                        self.refresh_data_async();
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::TrustDevices => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.discovered_devices.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        self.handle_trust_device();
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = WastelandView::Main;
                        self.selected_index = 0;
                        None
                    }
                    KeyCode::Char('r') => {
                        self.refresh_discovered_devices();
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::InstallRegistry => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.registry_modules.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        self.handle_install_module();
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = WastelandView::Main;
                        self.selected_index = 0;
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::ManageModules => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.installed_modules.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Char('a') => {
                        self.handle_archive_module();
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = WastelandView::Main;
                        self.selected_index = 0;
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::ArchivedModules => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.archived_modules.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        self.handle_restore_module();
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = WastelandView::Main;
                        self.selected_index = 0;
                        None
                    }
                    _ => None,
                }
            }
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        // Process any queued async messages first
        self.process_messages();

        // Update module bindings with current state
        module.config.bindings.insert(
            "current_view".to_string(),
            serde_json::json!(format!("{:?}", self.current_view)),
        );

        module.config.bindings.insert(
            "selected_index".to_string(),
            serde_json::json!(self.selected_index),
        );

        // Add status message if present
        if let Some(status) = &self.status_message {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(status),
            );
        } else {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(""),
            );
        }

        match self.current_view {
            WastelandView::Main => {
                let items = self.get_main_menu_items();
                module.config.bindings.insert(
                    "menu_items".to_string(),
                    serde_json::json!(items),
                );
            }
            WastelandView::TrustDevices => {
                let device_list: Vec<String> = self.discovered_devices
                    .iter()
                    .map(|(mac, name, rssi)| {
                        format!("{} ({}) - RSSI: {}", name, mac, rssi)
                    })
                    .collect();

                module.config.bindings.insert(
                    "device_list".to_string(),
                    serde_json::json!(device_list),
                );
            }
            WastelandView::InstallRegistry => {
                let module_list: Vec<String> = self.registry_modules
                    .iter()
                    .map(|m| format!("{} - {}", m.name, m.description))
                    .collect();

                module.config.bindings.insert(
                    "module_list".to_string(),
                    serde_json::json!(module_list),
                );
            }
            WastelandView::ManageModules => {
                module.config.bindings.insert(
                    "installed_modules".to_string(),
                    serde_json::json!(self.installed_modules),
                );
            }
            WastelandView::ArchivedModules => {
                module.config.bindings.insert(
                    "archived_modules".to_string(),
                    serde_json::json!(self.archived_modules),
                );
            }
        }
    }

    fn module_type(&self) -> &str {
        "wasteland_manager"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
