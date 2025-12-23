use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_bool_field(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>,
        key: &str,
        label: &str,
        field: EditorField,
    ) {
        if let Some(value) = bindings.get(key) {
            let bool_val = value.as_bool().unwrap_or(false);
            fields.push((label.to_string(), field, FieldValue::Bool(bool_val)));
        }
    }
}
