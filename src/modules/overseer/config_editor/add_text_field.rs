use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_text_field(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>,
        key: &str,
        label: &str,
        field: EditorField,
    ) {
        if let Some(value) = bindings.get(key) {
            let text = value.as_str().unwrap_or("").to_string();
            fields.push((label.to_string(), field, FieldValue::Text(text)));
        }
    }
}
