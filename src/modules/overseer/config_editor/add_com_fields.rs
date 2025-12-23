use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_com_fields(fields: &mut Vec<(String, EditorField, FieldValue)>, bindings: &HashMap<String, Value>) {
        Self::add_text_field(fields, bindings, "label", "Label", EditorField::Label);
        Self::add_text_field(fields, bindings, "toggle_on_label", "On Label", EditorField::ToggleOnLabel);
        Self::add_text_field(fields, bindings, "toggle_off_label", "Off Label", EditorField::ToggleOffLabel);
        Self::add_text_field(fields, bindings, "description", "Description", EditorField::Description);
        Self::add_bool_field(fields, bindings, "state", "State", EditorField::CustomBinding { key: "state".to_string() });
    }
}
