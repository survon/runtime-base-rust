use super::{ConfigEditor, EditorField};

impl ConfigEditor {
    /// Convert editor fields back to full module config
    pub fn to_full_config(&self, original_config: &serde_json::Value) -> serde_json::Value {
        let mut config = if self.is_new_module {
            // Start with empty config for new modules
            serde_json::json!({})
        } else {
            original_config.clone()
        };

        // Update base fields
        for (label, field, value) in &self.fields {
            match field {
                EditorField::Name => {
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("name".to_string(), value.to_json());
                    }
                }
                EditorField::BusTopic => {
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("bus_topic".to_string(), value.to_json());
                    }
                }
                EditorField::Template => {
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("template".to_string(), value.to_json());
                    }
                }
                EditorField::Model => {
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("model".to_string(), value.to_json());
                    }
                }
                EditorField::CustomBinding { key } if key == "module_type" => {
                    // Special case: module_type is a base field
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("module_type".to_string(), value.to_json());
                    }
                }
                _ => {
                    // These go in bindings section
                }
            }
        }

        // Update bindings
        let bindings = self.to_bindings();
        if let Some(obj) = config.as_object_mut() {
            obj.insert("bindings".to_string(), serde_json::to_value(bindings).unwrap());
        }

        config
    }
}
