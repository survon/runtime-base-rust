use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(in crate::module) fn add_number_field(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>,
        key: &str,
        label: &str,
        field: EditorField,
    ) {
        if let Some(value) = bindings.get(key) {
            let num = value.as_f64().unwrap_or(0.0);
            fields.push((label.to_string(), field, FieldValue::Number(num)));
        }
    }
}
