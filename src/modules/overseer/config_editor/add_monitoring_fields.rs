use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_monitoring_fields(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>
    ) {
        // Device metadata
        Self::add_text_field(fields, bindings, "device_id", "Device ID", EditorField::DeviceId);
        Self::add_text_field(fields, bindings, "display_name", "Display Name", EditorField::DisplayName);
        Self::add_text_field(fields, bindings, "unit_of_measure_label", "Unit Label", EditorField::UnitLabel);

        // Thresholds
        Self::add_number_field(fields, bindings, "max_value", "Max Value", EditorField::MaxValue);
        Self::add_number_field(fields, bindings, "warn_threshold", "Warning Threshold", EditorField::WarnThreshold);
        Self::add_number_field(fields, bindings, "danger_threshold", "Danger Threshold", EditorField::DangerThreshold);

        // Chart type (enum)
        if let Some(chart_type) = bindings.get("chart_type") {
            let current = chart_type.as_str().unwrap_or("line");
            let options = vec!["line".to_string(), "bar".to_string(), "sparkline".to_string()];
            let selected = options.iter().position(|o| o == current).unwrap_or(0);

            fields.push((
                "Chart Type".to_string(),
                EditorField::ChartType,
                FieldValue::Enum { options, selected },
            ));
        }

        // Boolean
        Self::add_bool_field(fields, bindings, "is_blinkable", "Blinkable", EditorField::IsBlinkable);
    }
}
