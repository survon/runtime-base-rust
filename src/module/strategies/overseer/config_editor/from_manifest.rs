use std::collections::HashMap;
use serde_json::Value;

use crate::module::get_supported_templates;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    /// Create editor from an existing manifest
    pub fn from_manifest(
        name: String,
        module_type: String,
        base_config: &serde_json::Value,
        bindings: &HashMap<String, Value>,
    ) -> Self {
        let mut fields = Vec::new();

        // === SECTION 1: Base Module Fields (ALL modules have these) ===

        // Name
        if let Some(name_val) = base_config.get("name").and_then(|v| v.as_str()) {
            fields.push((
                "Module Name".to_string(),
                EditorField::Name,
                FieldValue::Text(name_val.to_string()),
            ));
        }

        // Bus Topic
        if let Some(topic) = base_config.get("bus_topic").and_then(|v| v.as_str()) {
            fields.push((
                "Bus Topic".to_string(),
                EditorField::BusTopic,
                FieldValue::Text(topic.to_string()),
            ));
        }

        // Template (enum of supported templates)
        if let Some(template) = base_config.get("template").and_then(|v| v.as_str()) {
            let options = get_supported_templates()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let selected = options.iter().position(|o| o == template).unwrap_or(0);

            fields.push((
                "Template".to_string(),
                EditorField::Template,
                FieldValue::Enum { options, selected },
            ));
        }

        // === SECTION 2: Type-specific fields ===
        match module_type.as_str() {
            "monitoring" => {
                Self::add_monitoring_fields(&mut fields, bindings);
            }
            "valve_control" => {
                Self::add_valve_control_fields(&mut fields, bindings);
            }
            "llm" => {
                Self::add_llm_fields(&mut fields, bindings);

                // Add model field from base_config (special case for LLM)
                if let Some(model) = base_config.get("model").and_then(|v| v.as_str()) {
                    let options = vec!["search".to_string(), "summarizer".to_string(), "council".to_string()];
                    let selected = options.iter().position(|o| o == model).unwrap_or(0);

                    fields.push((
                        "Model".to_string(),
                        EditorField::Model,
                        FieldValue::Enum { options, selected },
                    ));
                }
            }
            "com" => {
                Self::add_com_fields(&mut fields, bindings);
            }
            _ => {
                // For unknown types, add all bindings as custom fields
            }
        }

        // === SECTION 3: Custom bindings (any not yet added) ===
        let existing_keys: std::collections::HashSet<String> = fields.iter()
            .filter_map(|(label, field, _)| {
                match field {
                    EditorField::DeviceId => Some("device_id".to_string()),
                    EditorField::DisplayName => Some("display_name".to_string()),
                    EditorField::UnitLabel => Some("unit_of_measure_label".to_string()),
                    EditorField::MaxValue => Some("max_value".to_string()),
                    EditorField::WarnThreshold => Some("warn_threshold".to_string()),
                    EditorField::DangerThreshold => Some("danger_threshold".to_string()),
                    EditorField::ChartType => Some("chart_type".to_string()),
                    EditorField::IsBlinkable => Some("is_blinkable".to_string()),
                    EditorField::Label => Some("label".to_string()),
                    EditorField::ToggleOnLabel => Some("toggle_on_label".to_string()),
                    EditorField::ToggleOffLabel => Some("toggle_off_label".to_string()),
                    EditorField::Description => Some("description".to_string()),
                    EditorField::Model => Some("model".to_string()),
                    EditorField::CustomBinding { key } => Some(key.clone()),
                    _ => None,
                }
            })
            .collect();

        for (key, value) in bindings {
            // Skip internal state fields (starting with _)
            if key.starts_with('_') {
                continue;
            }

            // Skip if already added
            if existing_keys.contains(key) {
                continue;
            }

            let field_value = match value {
                Value::String(s) => FieldValue::Text(s.clone()),
                Value::Number(n) => FieldValue::Number(n.as_f64().unwrap_or(0.0)),
                Value::Bool(b) => FieldValue::Bool(*b),
                _ => FieldValue::Text(value.to_string()),
            };

            fields.push((
                key.clone(),
                EditorField::CustomBinding { key: key.clone() },
                field_value,
            ));
        }

        Self {
            module_name: name,
            module_type,
            fields,
            selected_field: 0,
            is_editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,
            is_new_module: false,
        }
    }
}
