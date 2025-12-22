use crate::modules::{
    Module,
    wasteland_manager::{
        handler::WastelandManagerHandler,
        config_editor::{FieldValue},
    },
};

impl WastelandManagerHandler {
    pub(super) fn _update_bindings(&mut self, module: &mut Module) {
        self.process_messages();

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

        module.config.bindings.insert(
            "is_editing_config".to_string(),
            serde_json::json!(self.config_editor.is_some()),
        );

        if let Some(status) = &self.status_message {
            module
                .config
                .bindings
                .insert("status_message".to_string(), serde_json::json!(status));
        } else {
            module
                .config
                .bindings
                .insert("status_message".to_string(), serde_json::json!(""));
        }

        let pending_list: Vec<String> = self
            .pending_devices
            .iter()
            .map(|(mac, name, rssi)| format!("{} ({}) RSSI: {} dBm", name, mac, rssi))
            .collect();

        module.config.bindings.insert(
            "pending_devices".to_string(),
            serde_json::json!(pending_list),
        );

        let known_list: Vec<String> = self
            .known_devices
            .iter()
            .map(|device| {
                let trust_icon = if device.is_trusted { "✓" } else { "✗" };
                let rssi_str = device
                    .rssi
                    .map(|r| format!(" RSSI: {} dBm", r))
                    .unwrap_or_default();

                format!(
                    "{} {} ({}){}",
                    trust_icon, device.device_name, device.mac_address, rssi_str
                )
            })
            .collect();

        module
            .config
            .bindings
            .insert("known_devices".to_string(), serde_json::json!(known_list));

        let module_list: Vec<String> = self
            .registry_modules
            .iter()
            .map(|m| format!("{} - {}", m.name, m.description))
            .collect();

        module
            .config
            .bindings
            .insert("module_list".to_string(), serde_json::json!(module_list));

        module.config.bindings.insert(
            "installed_modules".to_string(),
            serde_json::json!(self.installed_modules),
        );

        module.config.bindings.insert(
            "archived_modules".to_string(),
            serde_json::json!(self.archived_modules),
        );

        if let Some(editor) = &self.config_editor {
            module.config.bindings.insert(
                "editor_module_name".to_string(),
                serde_json::json!(editor.module_name),
            );

            module.config.bindings.insert(
                "editor_module_type".to_string(),
                serde_json::json!(editor.module_type),
            );

            module.config.bindings.insert(
                "editor_selected_field".to_string(),
                serde_json::json!(editor.selected_field),
            );

            module.config.bindings.insert(
                "editor_is_editing".to_string(),
                serde_json::json!(editor.is_editing),
            );

            let fields_json: Vec<serde_json::Value> = editor
                .fields
                .iter()
                .map(|(label, _field, value)| {
                    serde_json::json!({
                        "label": label,
                        "value_type": match value {
                            FieldValue::Text(_) => "text",
                            FieldValue::Number(_) => "number",
                            FieldValue::Bool(_) => "bool",
                            FieldValue::Enum { .. } => "enum",
                        },
                        "display_value": value.as_display_string(),
                        "bool_value": match value {
                            FieldValue::Bool(b) => Some(*b),
                            _ => None,
                        },
                        "enum_options": match value {
                            FieldValue::Enum { options, .. } => Some(options.clone()),
                            _ => None,
                        },
                        "enum_selected": match value {
                            FieldValue::Enum { selected, .. } => Some(*selected),
                            _ => None,
                        },
                    })
                })
                .collect();

            module
                .config
                .bindings
                .insert("editor_fields".to_string(), serde_json::json!(fields_json));

            if editor.is_editing {
                module.config.bindings.insert(
                    "editor_edit_buffer".to_string(),
                    serde_json::json!(editor.edit_buffer),
                );

                module.config.bindings.insert(
                    "editor_cursor_pos".to_string(),
                    serde_json::json!(editor.cursor_pos),
                );
            } else {
                module
                    .config
                    .bindings
                    .insert("editor_edit_buffer".to_string(), serde_json::json!(""));

                module
                    .config
                    .bindings
                    .insert("editor_cursor_pos".to_string(), serde_json::json!(0));
            }
        } else {
            module.config.bindings.remove("editor_module_name");
            module.config.bindings.remove("editor_module_type");
            module.config.bindings.remove("editor_selected_field");
            module.config.bindings.remove("editor_is_editing");
            module.config.bindings.remove("editor_fields");
            module.config.bindings.remove("editor_edit_buffer");
            module.config.bindings.remove("editor_cursor_pos");
        }
    }
}
