use super::{
    ConfigEditor,
    EditorField,
    FieldValue,
};

impl ConfigEditor {
    /// Create a new module from scratch - starts with module type selection
    /// TODO possibly rename this to new_manifest
    pub fn new_module() -> Self {
        let module_types = vec![
            "monitoring".to_string(),
            "valve_control".to_string(),
            "com".to_string(),
            "llm".to_string(),
        ];

        let mut fields = Vec::new();

        // First field: Module Name
        fields.push((
            "Module Name".to_string(),
            EditorField::Name,
            FieldValue::Text("New Module".to_string()),
        ));

        // Second field: Module Type (enum)
        fields.push((
            "Module Type".to_string(),
            EditorField::CustomBinding { key: "module_type".to_string() },
            FieldValue::Enum { options: module_types, selected: 0 },
        ));

        Self {
            module_name: "new_module".to_string(),
            module_type: "monitoring".to_string(),
            fields,
            selected_field: 0,
            is_editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,
            is_new_module: true,
        }
    }
}
