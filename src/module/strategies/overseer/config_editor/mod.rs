mod field_value;
mod new_module;
mod expand_fields_for_type;
mod add_monitoring_defaults;
mod add_valve_control_defaults;
mod add_com_defaults;
mod add_llm_defaults;
mod from_manifest;
mod add_monitoring_fields;
mod add_valve_control_fields;
mod add_llm_fields;
mod add_com_fields;
mod add_text_field;
mod add_number_field;
mod add_bool_field;
mod to_full_config;
mod handle_key;
mod start_editing;
mod apply_edit;
mod to_bindings;
mod render;

use ratatui::{
    prelude::*,
    widgets::*,
};

pub use field_value::*;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorField {
    // Base fields (all modules)
    Name,
    BusTopic,
    Template,

    // Monitoring specific
    DeviceId,
    DisplayName,
    UnitLabel,
    MaxValue,
    WarnThreshold,
    DangerThreshold,
    ChartType,
    IsBlinkable,

    // Valve control specific
    Label,
    ToggleOnLabel,
    ToggleOffLabel,
    Description,

    // LLM specific
    Model,

    // Generic binding
    CustomBinding { key: String },
}

#[derive(Debug)]
pub struct ConfigEditor {
    pub module_name: String,
    pub module_type: String,
    pub fields: Vec<(String, EditorField, FieldValue)>,
    pub selected_field: usize,
    pub is_editing: bool,
    pub edit_buffer: String,
    pub cursor_pos: usize,
    pub is_new_module: bool, // Track if this is a new module being created
}

#[derive(Debug, PartialEq)]
pub enum EditorAction {
    None,
    ValueChanged,
    Save,
    Close,
    ModuleTypeSelected, // New action for when module type is picked
}
