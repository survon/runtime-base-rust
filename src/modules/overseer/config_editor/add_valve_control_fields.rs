use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_valve_control_fields(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>
    ) {
        Self::add_text_field(fields, bindings, "device_id", "Device ID", EditorField::DeviceId);
        Self::add_text_field(fields, bindings, "label", "Label", EditorField::Label);
        Self::add_text_field(fields, bindings, "toggle_on_label", "Open Label", EditorField::ToggleOnLabel);
        Self::add_text_field(fields, bindings, "toggle_off_label", "Closed Label", EditorField::ToggleOffLabel);
        Self::add_text_field(fields, bindings, "description", "Description", EditorField::Description);
    }
}
