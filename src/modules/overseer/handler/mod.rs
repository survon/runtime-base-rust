// src/modules/overseer/handler.rs
mod installer;
mod list_registry_modules;
mod fetch_registry_modules;
mod handle_key;
mod update_bindings;
mod trait_module_handler;
mod update_module_config;
mod archive_module;
mod list_archived_modules;
mod restore_module;
mod start_device_listener;
mod process_messages;
mod handle_scan_devices;
mod refresh_data;
mod refresh_known_devices;
mod handle_main_menu_select;
mod refresh_installed_modules;
mod handle_trust_device;
mod handle_ignore_device;
mod handle_toggle_trust;
mod handle_delete_device;
mod handle_install_module;
mod handle_archive_module;
mod handle_restore_module;
mod handle_manage_modules_enter;
mod get_config_editor;
mod handle_config_editor_save;
mod trigger_module_refresh;
mod new;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    prelude::*,
};
use std::{
    any::Any,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

use crate::{log_debug, log_error, log_info};
use crate::modules::{
    module_handler::ModuleHandler,
    overseer::{
        config_editor::{ConfigEditor, EditorAction, FieldValue},
        database::{KnownDevice, WastelandDatabase},
        handler::{installer::*},
    },
    ConfigValidator, Module,
};
use crate::util::{
    database::Database,
    io::{bus::MessageBus, discovery::DiscoveryManager, event::AppEvent},
};

/// Registry response format for module listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub module_type: String,
    pub template: String,
    pub download_url: String,
    pub checksum: String,
}

/// Registry API response
#[derive(Debug, Deserialize)]
pub struct RegistryResponse {
    pub modules: Vec<RegistryModule>,
    pub total: usize,
}

/// Module installation source
#[derive(Debug, Clone)]
pub enum InstallSource {
    Registry(String),         // module_id from registry
    LocalFile(PathBuf),       // Path to config.yml
    DiscoveredDevice(String), // MAC address of BLE device
}

#[derive(Debug, Clone)]
enum HandlerMessage {
    DevicesRefreshed(Vec<(String, String, i16)>),
    TrustedDevicesRefreshed(Vec<(String, String)>),
    KnownDevicesRefreshed(Vec<KnownDevice>),
    RegistryRefreshed(Vec<RegistryModule>),
    DeviceTrusted(String), // mac address
    DeviceDiscovered {
        mac: String,
        name: String,
        rssi: i16,
    },
    ModuleInstalled(String),
    OperationInProgress(String), // status message
    ScanProgress(u8),            // countdown in seconds
    ScanComplete(usize),
    ScanFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
enum WastelandView {
    Main,
    PendingTrust,
    AllDevices,
    InstallRegistry,
    ManageModules,
    ArchivedModules,
    EditConfig,
    CreateNewModule
}

#[derive(Debug)]
pub struct OverseerHandler {
    current_view: WastelandView,
    selected_index: usize,
    wasteland_path: PathBuf,
    archive_path: PathBuf,
    registry_url: String,
    discovery_manager: Option<Arc<DiscoveryManager>>,
    database: Database,
    message_bus: MessageBus,
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,
    pending_devices: Vec<(String, String, i16)>, // (mac, name, rssi)
    known_devices: Vec<KnownDevice>,
    registry_modules: Vec<RegistryModule>,
    installed_modules: Vec<String>,
    archived_modules: Vec<String>,
    status_message: Option<String>,
    is_scanning: bool,
    scan_countdown: u8,
    config_editor: Option<ConfigEditor>,
}
