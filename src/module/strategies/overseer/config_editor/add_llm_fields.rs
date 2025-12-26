use std::collections::HashMap;
use serde_json::Value;

use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(in crate::module) fn add_llm_fields(
        fields: &mut Vec<(String, EditorField, FieldValue)>,
        bindings: &HashMap<String, Value>
    ) {
        // LLM modules don't have many bindings fields to edit
        // The model field is handled in the caller since it's in base_config
        // Most LLM fields are runtime state (chat_history, etc.) that shouldn't be edited
    }
}
