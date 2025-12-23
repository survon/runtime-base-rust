use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_monitoring_defaults(&mut self) {
        self.fields.push((
            "Device ID".to_string(),
            EditorField::DeviceId,
            FieldValue::Text("device_001".to_string()),
        ));

        self.fields.push((
            "Display Name".to_string(),
            EditorField::DisplayName,
            FieldValue::Text("Sensor".to_string()),
        ));

        self.fields.push((
            "Unit Label".to_string(),
            EditorField::UnitLabel,
            FieldValue::Text("PSI".to_string()),
        ));

        self.fields.push((
            "Max Value".to_string(),
            EditorField::MaxValue,
            FieldValue::Number(100.0),
        ));

        self.fields.push((
            "Warning Threshold".to_string(),
            EditorField::WarnThreshold,
            FieldValue::Number(75.0),
        ));

        self.fields.push((
            "Danger Threshold".to_string(),
            EditorField::DangerThreshold,
            FieldValue::Number(90.0),
        ));

        let chart_options = vec!["line".to_string(), "bar".to_string(), "sparkline".to_string()];
        self.fields.push((
            "Chart Type".to_string(),
            EditorField::ChartType,
            FieldValue::Enum { options: chart_options, selected: 0 },
        ));

        self.fields.push((
            "Blinkable".to_string(),
            EditorField::IsBlinkable,
            FieldValue::Bool(true),
        ));
    }
}
