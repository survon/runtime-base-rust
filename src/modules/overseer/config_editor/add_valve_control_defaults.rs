use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_valve_control_defaults(&mut self) {
        self.fields.push((
            "Device ID".to_string(),
            EditorField::DeviceId,
            FieldValue::Text("valve_001".to_string()),
        ));

        self.fields.push((
            "Label".to_string(),
            EditorField::Label,
            FieldValue::Text("Valve Control".to_string()),
        ));

        self.fields.push((
            "Open Label".to_string(),
            EditorField::ToggleOnLabel,
            FieldValue::Text("Open".to_string()),
        ));

        self.fields.push((
            "Closed Label".to_string(),
            EditorField::ToggleOffLabel,
            FieldValue::Text("Closed".to_string()),
        ));

        self.fields.push((
            "Description".to_string(),
            EditorField::Description,
            FieldValue::Text("Controls valve state".to_string()),
        ));
    }
}
