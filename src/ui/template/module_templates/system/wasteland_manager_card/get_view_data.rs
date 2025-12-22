use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Color,

};

use crate::modules::Module;
use super::{ViewData, WastelandManagerCard};

impl WastelandManagerCard {
    pub(super) fn get_view_data<'a>(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &'a mut Module
    ) -> ViewData<'a> {
        let current_view = module
            .config
            .bindings
            .get("current_view")
            .and_then(|v| v.as_str())
            .unwrap_or("Main");

        let selected_index = module
            .config
            .bindings
            .get("selected_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let status_message = module
            .config
            .bindings
            .get("status_message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let known_count = module
            .config
            .bindings
            .get("known_devices")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let registry_count = module
            .config
            .bindings
            .get("module_list")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let known_devices = module
            .config
            .bindings
            .get("known_devices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let installed_count = module
            .config
            .bindings
            .get("installed_modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let archived_count = module
            .config
            .bindings
            .get("archived_modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let is_scanning = module
            .config
            .bindings
            .get("is_scanning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scan_countdown = module
            .config
            .bindings
            .get("scan_countdown")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let _pending_devices_arr = module
            .config
            .bindings
            .get("pending_devices")
            .and_then(|v| v.as_array());

        let pending_count = _pending_devices_arr
            .map(|arr| arr.len())
            .unwrap_or(0);

        let pending_devices = _pending_devices_arr
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let module_list = module
            .config
            .bindings
            .get("module_list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let installed_modules = module
            .config
            .bindings
            .get("installed_modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let has_status = status_message.is_some();

        let border_color = if is_selected { Color::White } else { Color::Cyan };

        ViewData {
            current_view,
            selected_index,
            status_message,
            border_color,
            module_list,
            installed_modules,
            known_devices,
            pending_devices,
            pending_count,
            known_count,
            registry_count,
            installed_count,
            archived_count,
            is_scanning,
            scan_countdown,
            has_status,
        }
    }
}
