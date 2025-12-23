use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField};

impl ConfigEditor {
    pub fn to_bindings(&self) -> HashMap<String, Value> {
        let mut bindings = HashMap::new();

        for (label, field, value) in &self.fields {
            let key = match field {
                EditorField::DeviceId => "device_id",
                EditorField::DisplayName => "display_name",
                EditorField::UnitLabel => "unit_of_measure_label",
                EditorField::MaxValue => "max_value",
                EditorField::WarnThreshold => "warn_threshold",
                EditorField::DangerThreshold => "danger_threshold",
                EditorField::ChartType => "chart_type",
                EditorField::IsBlinkable => "is_blinkable",
                EditorField::Label => "label",
                EditorField::ToggleOnLabel => "toggle_on_label",
                EditorField::ToggleOffLabel => "toggle_off_label",
                EditorField::Description => "description",
                EditorField::Model => "model",
                EditorField::CustomBinding { key } => {
                    // Skip module_type as it goes in base config
                    if key == "module_type" {
                        continue;
                    }
                    key.as_str()
                }
                _ => continue,
            };

            bindings.insert(key.to_string(), value.to_json());
        }

        bindings
    }
}
