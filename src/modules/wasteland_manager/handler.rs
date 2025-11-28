// src/modules/wasteland_manager/handler.rs
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::{log_debug,log_error};
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::{
    io::{
        event::AppEvent,
        discovery::DiscoveryManager,
        bus::MessageBus,
    },
    wasteland_manager::WastelandManager,
    database::Database
};

#[derive(Debug, Clone)]
enum HandlerMessage {
    DevicesRefreshed(Vec<(String, String, i16)>),
    TrustedDevicesRefreshed(Vec<(String, String)>),
    KnownDevicesRefreshed(Vec<crate::util::database::KnownDevice>),
    RegistryRefreshed(Vec<crate::util::wasteland_manager::RegistryModule>),
    DeviceTrusted(String), // mac address
    DeviceDiscovered { mac: String, name: String, rssi: i16 },
    ModuleInstalled(String),
    OperationInProgress(String), // status message
    ScanProgress(u8), // countdown in seconds
    ScanComplete(usize),
    ScanFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
enum WastelandView {
    Main,              // Show menu options
    PendingTrust,      // List pending devices to trust
    AllDevices,        // Manage all known devices
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
    database: Database,
    message_bus: MessageBus,
    // Channel for async updates
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,
    // Cached data
    pending_devices: Vec<(String, String, i16)>, // (mac, name, rssi)
    known_devices: Vec<crate::util::database::KnownDevice>,
    registry_modules: Vec<crate::util::wasteland_manager::RegistryModule>,
    installed_modules: Vec<String>,
    archived_modules: Vec<String>,
    // Status message
    status_message: Option<String>,
    is_scanning: bool,
    scan_countdown: u8,
}

impl WastelandManagerHandler {
    pub fn new(
        wasteland_path: std::path::PathBuf,
        discovery_manager: Option<Arc<DiscoveryManager>>,
        database: Database,
        message_bus: MessageBus,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            current_view: WastelandView::Main,
            selected_index: 0,
            wasteland_manager: WastelandManager::new(wasteland_path.clone()),
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
        };

        // Start listening for device discovery events
        handler.start_device_listener();

        // Do initial async refresh
        handler.refresh_data_async();
        handler
    }

    fn start_device_listener(&mut self) {
        let tx = self.message_tx.clone();

        // Fix 1: Clone the bus so we can move it into the async block safely.
        // This avoids lifetime issues with 'self' and the future.
        let bus = self.message_bus.clone();

        tokio::spawn(async move {
            // Fix 2: Remove the match/Ok/Err.
            // .subscribe() returns the receiver directly.
            let mut receiver = bus.subscribe("device_discovered".to_string()).await;

            // Fix 3: changed 'Ok(msg)' to 'Some(msg)'
            // Tokio mpsc receivers return None when closed, not Err.
            while let Some(msg) = receiver.recv().await {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    if data.get("requires_trust_decision")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        let mac = data["mac_address"].as_str().unwrap_or("").to_string();
                        let name = data["name"].as_str().unwrap_or("Unknown").to_string();
                        let rssi = data["rssi"].as_i64().unwrap_or(0) as i16;

                        let _ = tx.send(HandlerMessage::DeviceDiscovered { mac, name, rssi });
                    }
                }
            }
        });
    }

    fn process_messages(&mut self) {
        // Non-blocking: process all queued messages
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::DevicesRefreshed(devices) => {
                    // These are discovered (not yet in DB) devices
                    self.pending_devices = devices;
                    self.status_message = None;
                }
                HandlerMessage::KnownDevicesRefreshed(devices) => {
                    self.known_devices = devices;
                    self.status_message = None;
                }
                HandlerMessage::RegistryRefreshed(modules) => {
                    self.registry_modules = modules;
                    self.status_message = None;
                }
                HandlerMessage::DeviceTrusted(mac) => {
                    self.pending_devices.retain(|(m, _, _)| m != &mac);
                    if self.selected_index > 0 && self.selected_index >= self.pending_devices.len() {
                        self.selected_index -= 1;
                    }
                    self.status_message = Some(format!("✓ Device {} trusted!", mac));
                    self.refresh_known_devices();
                }
                HandlerMessage::DeviceDiscovered { mac, name, rssi } => {
                    // Add to pending list if not already there
                    if !self.pending_devices.iter().any(|(m, _, _)| m == &mac) {
                        self.pending_devices.push((mac, name, rssi));
                    }
                }
                HandlerMessage::ModuleInstalled(name) => {
                    self.refresh_installed_modules();
                    self.status_message = Some(format!("✓ Module {} installed!", name));
                }
                HandlerMessage::OperationInProgress(msg) => {
                    self.status_message = Some(msg);
                }
                HandlerMessage::ScanProgress(seconds) => {
                    self.scan_countdown = seconds;
                    self.status_message = Some(format!("🔍 Scanning... {} seconds remaining", seconds));
                }
                HandlerMessage::ScanComplete(count) => {
                    self.is_scanning = false;
                    self.scan_countdown = 0;
                    self.status_message = Some(format!("✅ Scan complete! {} new device(s) found", count));
                    self.refresh_data_async();
                }
                HandlerMessage::ScanFailed(err_msg) => {
                    self.is_scanning = false;
                    self.scan_countdown = 0;
                    self.status_message = Some(err_msg);
                }
                _ => {}
            }
        }
    }

    fn handle_scan_devices(&mut self) {
        if self.is_scanning {
            self.status_message = Some("⚠️ Scan already in progress".to_string());
            return;
        }

        if let Some(discovery) = &self.discovery_manager {
            self.is_scanning = true;
            let scan_duration = 15; // Increased to 15s as requested
            self.scan_countdown = scan_duration as u8;

            let discovery_clone = discovery.clone();
            let tx = self.message_tx.clone();

            tokio::spawn(async move {
                // Create the visual countdown task
                let countdown_task = async {
                    for i in (1..=scan_duration).rev() {
                        let _ = tx.send(HandlerMessage::ScanProgress(i as u8));
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                };

                // Create the actual scan task
                // Note: We pass the duration to scan_once now
                let scan_task = async {
                    discovery_clone.scan_once(scan_duration).await
                };

                // Run them concurrently!
                // We ignore the output of countdown_task, we only care about scan_result
                let (_, scan_result) = tokio::join!(countdown_task, scan_task);

                match scan_result {
                    Ok(count) => {
                        let _ = tx.send(HandlerMessage::ScanComplete(count));
                    }
                    Err(e) => {
                        // Send ScanFailed so the flag gets reset
                        let _ = tx.send(HandlerMessage::ScanFailed(
                            format!("❌ Scan failed: {}", e)
                        ));
                    }
                }
            });

            self.status_message = Some("🔍 Starting BLE scan...".to_string());
        } else {
            self.status_message = Some("❌ Discovery manager not available".to_string());
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
        }

        // Refresh known devices from database
        self.refresh_known_devices();

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

    fn refresh_known_devices(&mut self) {
        let tx = self.message_tx.clone();
        let db = self.database.clone();

        tokio::spawn(async move {
            if let Ok(devices) = db.get_all_known_devices() {
                let _ = tx.send(HandlerMessage::KnownDevicesRefreshed(devices));
            }
        });
    }

    fn handle_main_menu_select(&mut self) {
        match self.selected_index {
            0 => {
                self.current_view = WastelandView::PendingTrust;
                self.selected_index = 0;
            }
            1 => {
                self.current_view = WastelandView::AllDevices;
                self.selected_index = 0;
                self.refresh_known_devices();
            }
            2 => {
                self.current_view = WastelandView::InstallRegistry;
                self.selected_index = 0;
            }
            3 => {
                self.current_view = WastelandView::ManageModules;
                self.selected_index = 0;
                self.refresh_installed_modules();
            }
            4 => {
                self.current_view = WastelandView::ArchivedModules;
                self.selected_index = 0;
                self.refresh_data_async();
            }
            5 => {
                // Back - will be handled by returning AppEvent::Back
            }
            _ => {}
        }
    }

    fn refresh_installed_modules(&mut self) {
        use std::fs;

        let wasteland_path = std::path::PathBuf::from("./modules/wasteland/");
        self.installed_modules.clear();

        if let Ok(entries) = fs::read_dir(&wasteland_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
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
        if self.selected_index < self.pending_devices.len() {
            let (mac, name, _rssi) = &self.pending_devices[self.selected_index];
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

    fn handle_ignore_device(&mut self) {
        if self.selected_index < self.pending_devices.len() {
            self.pending_devices.remove(self.selected_index);
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
            self.status_message = Some("Device ignored".to_string());
        }
    }

    fn handle_toggle_trust(&mut self) {
        if self.selected_index < self.known_devices.len() {
            let device = &self.known_devices[self.selected_index];
            let mac = device.mac_address.clone();
            let new_trust = !device.is_trusted;

            if let Err(e) = self.database.set_device_trust(&mac, new_trust) {
                self.status_message = Some(format!("Failed to update trust: {}", e));
            } else {
                self.status_message = Some(if new_trust {
                    "Device trusted".to_string()
                } else {
                    "Device untrusted".to_string()
                });
                self.refresh_known_devices();
            }
        }
    }

    fn handle_delete_device(&mut self) {
        if self.selected_index < self.known_devices.len() {
            let device = &self.known_devices[self.selected_index];
            let mac = device.mac_address.clone();

            if let Err(e) = self.database.delete_device(&mac) {
                self.status_message = Some(format!("Failed to delete: {}", e));
            } else {
                self.status_message = Some("Device deleted".to_string());
                self.refresh_known_devices();
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
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
        log_debug!("handle_key: {:?}", key_code);
        match self.current_view {

            WastelandView::Main => {
                log_debug!("in main view...");
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        log_debug!("pressed Up. selected index: {}", self.selected_index);
                        None
                    }
                    KeyCode::Down => {
                        let max = 5; // 6 menu items (0-5)
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        log_debug!("pressed Up. selected index: {}", self.selected_index);
                        None
                    }
                    KeyCode::Enter => {
                        if self.selected_index == 5 {
                            Some(AppEvent::Back)
                        } else {
                            self.handle_main_menu_select();
                            None
                        }
                    }
                    KeyCode::Esc => Some(AppEvent::Back),
                    KeyCode::Char('r') => {
                        self.refresh_data_async();
                        None
                    }
                    KeyCode::Char('s') => {
                        self.handle_scan_devices();
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::PendingTrust => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.pending_devices.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        self.handle_trust_device();
                        None
                    }
                    KeyCode::Char('i') => {
                        self.handle_ignore_device();
                        None
                    }
                    KeyCode::Char('v') => {
                        self.current_view = WastelandView::AllDevices;
                        self.selected_index = 0;
                        self.refresh_known_devices();
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = WastelandView::Main;
                        self.selected_index = 0;
                        None
                    }
                    KeyCode::Char('r') => {
                        self.refresh_data_async();
                        None
                    }
                    KeyCode::Char('s') => {
                        self.handle_scan_devices();
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::AllDevices => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        let max = self.known_devices.len().saturating_sub(1);
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Char('t') => {
                        self.handle_toggle_trust();
                        None
                    }
                    KeyCode::Char('d') => {
                        self.handle_delete_device();
                        None
                    }
                    KeyCode::Char('p') => {
                        self.current_view = WastelandView::PendingTrust;
                        self.selected_index = 0;
                        None
                    }
                    KeyCode::Char('r') => {
                        self.refresh_known_devices();
                        None
                    }
                    KeyCode::Char('s') => {
                        self.handle_scan_devices();
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

        module.config.bindings.insert(
            "is_scanning".to_string(),
            serde_json::json!(self.is_scanning),
        );

        module.config.bindings.insert(
            "scan_countdown".to_string(),
            serde_json::json!(self.scan_countdown),
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

        // Update pending devices
        let pending_list: Vec<String> = self.pending_devices
            .iter()
            .map(|(mac, name, rssi)| {
                format!("{} ({}) RSSI: {} dBm", name, mac, rssi)
            })
            .collect();

        module.config.bindings.insert(
            "pending_devices".to_string(),
            serde_json::json!(pending_list),
        );

        // Update known devices
        let known_list: Vec<String> = self.known_devices
            .iter()
            .map(|device| {
                let trust_icon = if device.is_trusted { "✓" } else { "✗" };
                let rssi_str = device.rssi
                    .map(|r| format!(" RSSI: {} dBm", r))
                    .unwrap_or_default();

                format!(
                    "{} {} ({}){}",
                    trust_icon,
                    device.device_name,
                    device.mac_address,
                    rssi_str
                )
            })
            .collect();

        module.config.bindings.insert(
            "known_devices".to_string(),
            serde_json::json!(known_list),
        );

        // Update registry modules
        let module_list: Vec<String> = self.registry_modules
            .iter()
            .map(|m| format!("{} - {}", m.name, m.description))
            .collect();

        module.config.bindings.insert(
            "module_list".to_string(),
            serde_json::json!(module_list),
        );

        // Update installed modules
        module.config.bindings.insert(
            "installed_modules".to_string(),
            serde_json::json!(self.installed_modules),
        );

        // Update archived modules
        module.config.bindings.insert(
            "archived_modules".to_string(),
            serde_json::json!(self.archived_modules),
        );
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
