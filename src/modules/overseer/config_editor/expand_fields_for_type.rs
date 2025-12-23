use crate::modules::get_supported_templates;

use super::{
    ConfigEditor,
    EditorField,
    FieldValue,
};

impl ConfigEditor {
    /// After module type is selected, expand fields for that type
    pub(super) fn expand_fields_for_type(&mut self, module_type: &str) {
        self.module_type = module_type.to_string();

        // Keep existing Name and Module Type fields
        let name = self.fields[0].2.as_display_string();

        self.fields.clear();

        // Re-add name
        self.fields.push((
            "Module Name".to_string(),
            EditorField::Name,
            FieldValue::Text(name),
        ));

        // Add Bus Topic (auto-generated from name)
        let bus_topic = self.module_name.to_lowercase().replace(" ", "_");
        self.fields.push((
            "Bus Topic".to_string(),
            EditorField::BusTopic,
            FieldValue::Text(bus_topic),
        ));

        // Add Template field
        let templates = get_supported_templates();
        let template_options: Vec<String> = templates.iter().map(|s| s.to_string()).collect();
        let default_template = match module_type {
            "monitoring" => "gauge_card",
            "valve_control" | "com" => "toggle_switch",
            "llm" => "chat_interface",
            _ => "gauge_card",
        };
        let selected = template_options.iter()
            .position(|t| t == default_template)
            .unwrap_or(0);

        self.fields.push((
            "Template".to_string(),
            EditorField::Template,
            FieldValue::Enum { options: template_options, selected },
        ));

        // Add type-specific fields with default values
        match module_type {
            "monitoring" => self.add_monitoring_defaults(),
            "valve_control" => self.add_valve_control_defaults(),
            "com" => self.add_com_defaults(),
            "llm" => self.add_llm_defaults(),
            _ => {}
        }
    }
}
