// src/modules/overseer/config_editor.rs
// Interactive TUI form for editing module configurations

use ratatui::{
    prelude::*,
    widgets::*,
};
use crossterm::event::KeyCode;
use serde_json::Value;
use std::collections::HashMap;

use crate::modules::config_schema::*;

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

#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Enum { options: Vec<String>, selected: usize },
}

impl FieldValue {
    pub fn as_display_string(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(n) => format!("{}", n),
            Self::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Self::Enum { options, selected } => {
                options.get(*selected).cloned().unwrap_or_default()
            }
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Self::Text(s) => Value::String(s.clone()),
            Self::Number(n) => serde_json::json!(n),
            Self::Bool(b) => Value::Bool(*b),
            Self::Enum { options, selected } => {
                Value::String(options.get(*selected).cloned().unwrap_or_default())
            }
        }
    }
}

impl ConfigEditor {
    /// Create a new module from scratch - starts with module type selection
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

    /// After module type is selected, expand fields for that type
    pub fn expand_fields_for_type(&mut self, module_type: &str) {
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

    fn add_monitoring_defaults(&mut self) {
        self.fields.push((
            "Device ID".to_string(),
            EditorField::DeviceId,
            FieldValue::Text("device_001".to_string()),
        ));

        self.fields.push((
            "Display Name".to_string(),
            EditorField::DisplayName,
            FieldValue::Text("Sensor".to_string()),
        ));

        self.fields.push((
            "Unit Label".to_string(),
            EditorField::UnitLabel,
            FieldValue::Text("PSI".to_string()),
        ));

        self.fields.push((
            "Max Value".to_string(),
            EditorField::MaxValue,
            FieldValue::Number(100.0),
        ));

        self.fields.push((
            "Warning Threshold".to_string(),
            EditorField::WarnThreshold,
            FieldValue::Number(75.0),
        ));

        self.fields.push((
            "Danger Threshold".to_string(),
            EditorField::DangerThreshold,
            FieldValue::Number(90.0),
        ));

        let chart_options = vec!["line".to_string(), "bar".to_string(), "sparkline".to_string()];
        self.fields.push((
            "Chart Type".to_string(),
            EditorField::ChartType,
            FieldValue::Enum { options: chart_options, selected: 0 },
        ));

        self.fields.push((
            "Blinkable".to_string(),
            EditorField::IsBlinkable,
            FieldValue::Bool(true),
        ));
    }

    fn add_valve_control_defaults(&mut self) {
        self.fields.push((
            "Device ID".to_string(),
            EditorField::DeviceId,
            FieldValue::Text("valve_001".to_string()),
        ));

        self.fields.push((
            "Label".to_string(),
            EditorField::Label,
            FieldValue::Text("Valve Control".to_string()),
        ));

        self.fields.push((
            "Open Label".to_string(),
            EditorField::ToggleOnLabel,
            FieldValue::Text("Open".to_string()),
        ));

        self.fields.push((
            "Closed Label".to_string(),
            EditorField::ToggleOffLabel,
            FieldValue::Text("Closed".to_string()),
        ));

        self.fields.push((
            "Description".to_string(),
            EditorField::Description,
            FieldValue::Text("Controls valve state".to_string()),
        ));
    }

    fn add_com_defaults(&mut self) {
        self.fields.push((
            "Label".to_string(),
            EditorField::Label,
            FieldValue::Text("Device Control".to_string()),
        ));

        self.fields.push((
            "On Label".to_string(),
            EditorField::ToggleOnLabel,
            FieldValue::Text("On".to_string()),
        ));

        self.fields.push((
            "Off Label".to_string(),
            EditorField::ToggleOffLabel,
            FieldValue::Text("Off".to_string()),
        ));

        self.fields.push((
            "Description".to_string(),
            EditorField::Description,
            FieldValue::Text("Remote device control".to_string()),
        ));

        self.fields.push((
            "State".to_string(),
            EditorField::CustomBinding { key: "state".to_string() },
            FieldValue::Bool(false),
        ));
    }

    fn add_llm_defaults(&mut self) {
        let model_options = vec!["search".to_string(), "summarizer".to_string(), "council".to_string()];
        self.fields.push((
            "Model".to_string(),
            EditorField::Model,
            FieldValue::Enum { options: model_options, selected: 0 },
        ));
    }

    /// Create editor from an existing module's config
    pub fn from_module_config(
        name: String,
        module_type: String,
        base_config: &serde_json::Value,
        bindings: &HashMap<String, Value>,
    ) -> Self {
        let mut fields = Vec::new();

        // === SECTION 1: Base Module Fields (ALL modules have these) ===

        // Name
        if let Some(name_val) = base_config.get("name").and_then(|v| v.as_str()) {
            fields.push((
                "Module Name".to_string(),
                EditorField::Name,
                FieldValue::Text(name_val.to_string()),
            ));
        }

        // Bus Topic
        if let Some(topic) = base_config.get("bus_topic").and_then(|v| v.as_str()) {
            fields.push((
                "Bus Topic".to_string(),
                EditorField::BusTopic,
                FieldValue::Text(topic.to_string()),
            ));
        }

        // Template (enum of supported templates)
        if let Some(template) = base_config.get("template").and_then(|v| v.as_str()) {
            let options = get_supported_templates()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let selected = options.iter().position(|o| o == template).unwrap_or(0);

            fields.push((
                "Template".to_string(),
                EditorField::Template,
                FieldValue::Enum { options, selected },
            ));
        }

        // === SECTION 2: Type-specific fields ===
        match module_type.as_str() {
            "monitoring" => {
                Self::add_monitoring_fields(&mut fields, bindings);
            }
            "valve_control" => {
                Self::add_valve_control_fields(&mut fields, bindings);
            }
            "llm" => {
                Self::add_llm_fields(&mut fields, bindings);

                // Add model field from base_config (special case for LLM)
                if let Some(model) = base_config.get("model").and_then(|v| v.as_str()) {
                    let options = vec!["search".to_string(), "summarizer".to_string(), "council".to_string()];
                    let selected = options.iter().position(|o| o == model).unwrap_or(0);

                    fields.push((
                        "Model".to_string(),
                        EditorField::Model,
                        FieldValue::Enum { options, selected },
                    ));
                }
            }
            "com" => {
                Self::add_com_fields(&mut fields, bindings);
            }
            _ => {
                // For unknown types, add all bindings as custom fields
            }
        }

        // === SECTION 3: Custom bindings (any not yet added) ===
        let existing_keys: std::collections::HashSet<String> = fields.iter()
            .filter_map(|(label, field, _)| {
                match field {
                    EditorField::DeviceId => Some("device_id".to_string()),
                    EditorField::DisplayName => Some("display_name".to_string()),
                    EditorField::UnitLabel => Some("unit_of_measure_label".to_string()),
                    EditorField::MaxValue => Some("max_value".to_string()),
                    EditorField::WarnThreshold => Some("warn_threshold".to_string()),
                    EditorField::DangerThreshold => Some("danger_threshold".to_string()),
                    EditorField::ChartType => Some("chart_type".to_string()),
                    EditorField::IsBlinkable => Some("is_blinkable".to_string()),
                    EditorField::Label => Some("label".to_string()),
                    EditorField::ToggleOnLabel => Some("toggle_on_label".to_string()),
                    EditorField::ToggleOffLabel => Some("toggle_off_label".to_string()),
                    EditorField::Description => Some("description".to_string()),
                    EditorField::Model => Some("model".to_string()),
                    EditorField::CustomBinding { key } => Some(key.clone()),
                    _ => None,
                }
            })
            .collect();

        for (key, value) in bindings {
            // Skip internal state fields (starting with _)
            if key.starts_with('_') {
                continue;
            }

            // Skip if already added
            if existing_keys.contains(key) {
                continue;
            }

            let field_value = match value {
                Value::String(s) => FieldValue::Text(s.clone()),
                Value::Number(n) => FieldValue::Number(n.as_f64().unwrap_or(0.0)),
                Value::Bool(b) => FieldValue::Bool(*b),
                _ => FieldValue::Text(value.to_string()),
            };

            fields.push((
                key.clone(),
                EditorField::CustomBinding { key: key.clone() },
                field_value,
            ));
        }

        Self {
            module_name: name,
            module_type,
            fields,
            selected_field: 0,
            is_editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,
            is_new_module: false,
        }
    }

    fn add_monitoring_fields(fields: &mut Vec<(String, EditorField, FieldValue)>, bindings: &HashMap<String, Value>) {
        // Device metadata
        Self::add_text_field(fields, bindings, "device_id", "Device ID", EditorField::DeviceId);
        Self::add_text_field(fields, bindings, "display_name", "Display Name", EditorField::DisplayName);
        Self::add_text_field(fields, bindings, "unit_of_measure_label", "Unit Label", EditorField::UnitLabel);

        // Thresholds
        Self::add_number_field(fields, bindings, "max_value", "Max Value", EditorField::MaxValue);
        Self::add_number_field(fields, bindings, "warn_threshold", "Warning Threshold", EditorField::WarnThreshold);
        Self::add_number_field(fields, bindings, "danger_threshold", "Danger Threshold", EditorField::DangerThreshold);

        // Chart type (enum)
        if let Some(chart_type) = bindings.get("chart_type") {
            let current = chart_type.as_str().unwrap_or("line");
            let options = vec!["line".to_string(), "bar".to_string(), "sparkline".to_string()];
            let selected = options.iter().position(|o| o == current).unwrap_or(0);

            fields.push((
                "Chart Type".to_string(),
                EditorField::ChartType,
                FieldValue::Enum { options, selected },
            ));
        }

        // Boolean
        Self::add_bool_field(fields, bindings, "is_blinkable", "Blinkable", EditorField::IsBlinkable);
    }

    fn add_valve_control_fields(fields: &mut Vec<(String, EditorField, FieldValue)>, bindings: &HashMap<String, Value>) {
        Self::add_text_field(fields, bindings, "device_id", "Device ID", EditorField::DeviceId);
        Self::add_text_field(fields, bindings, "label", "Label", EditorField::Label);
        Self::add_text_field(fields, bindings, "toggle_on_label", "Open Label", EditorField::ToggleOnLabel);
        Self::add_text_field(fields, bindings, "toggle_off_label", "Closed Label", EditorField::ToggleOffLabel);
        Self::add_text_field(fields, bindings, "description", "Description", EditorField::Description);
    }

    fn add_llm_fields(fields: &mut Vec<(String, EditorField, FieldValue)>, bindings: &HashMap<String, Value>) {
        // LLM modules don't have many bindings fields to edit
        // The model field is handled in the caller since it's in base_config
        // Most LLM fields are runtime state (chat_history, etc.) that shouldn't be edited
    }

    fn add_com_fields(fields: &mut Vec<(String, EditorField, FieldValue)>, bindings: &HashMap<String, Value>) {
        Self::add_text_field(fields, bindings, "label", "Label", EditorField::Label);
        Self::add_text_field(fields, bindings, "toggle_on_label", "On Label", EditorField::ToggleOnLabel);
        Self::add_text_field(fields, bindings, "toggle_off_label", "Off Label", EditorField::ToggleOffLabel);
        Self::add_text_field(fields, bindings, "description", "Description", EditorField::Description);
        Self::add_bool_field(fields, bindings, "state", "State", EditorField::CustomBinding { key: "state".to_string() });
    }

    fn add_text_field(
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

    fn add_number_field(
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

    fn add_bool_field(
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

    pub fn handle_key(&mut self, key: KeyCode) -> EditorAction {
        // Special handling for new modules in initial setup
        if self.is_new_module && self.fields.len() == 2 {
            // We're in the initial "pick module type" phase
            match key {
                KeyCode::Up => {
                    if self.selected_field > 0 {
                        self.selected_field -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Down => {
                    if self.selected_field < 1 {
                        self.selected_field += 1;
                    }
                    EditorAction::None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if self.selected_field == 1 {
                        // User is on Module Type field - expand with that type
                        if let Some((_, _, FieldValue::Enum { options, selected })) = self.fields.get(1) {
                            let module_type = options[*selected].clone();
                            self.expand_fields_for_type(&module_type);
                            self.selected_field = 0;
                            return EditorAction::ModuleTypeSelected;
                        }
                    } else {
                        // On Name field - allow editing
                        self.start_editing();
                    }
                    EditorAction::None
                }
                KeyCode::Left | KeyCode::Right => {
                    // For module type enum, cycle through options
                    if self.selected_field == 1 {
                        if let Some((_, _, value)) = self.fields.get_mut(1) {
                            if let FieldValue::Enum { options, selected } = value {
                                if key == KeyCode::Right {
                                    *selected = (*selected + 1) % options.len();
                                } else if *selected > 0 {
                                    *selected -= 1;
                                } else {
                                    *selected = options.len() - 1;
                                }
                                return EditorAction::ValueChanged;
                            }
                        }
                    }
                    EditorAction::None
                }
                KeyCode::Esc => EditorAction::Close,
                _ => EditorAction::None,
            }
        } else if self.is_editing {
            match key {
                KeyCode::Esc => {
                    self.is_editing = false;
                    self.edit_buffer.clear();
                    EditorAction::None
                }
                KeyCode::Enter => {
                    self.apply_edit();
                    self.is_editing = false;

                    // Update module_name if we edited the name field
                    if let Some((_, EditorField::Name, value)) = self.fields.get(0) {
                        self.module_name = value.as_display_string()
                            .to_lowercase()
                            .replace(" ", "_");
                    }

                    EditorAction::ValueChanged
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    EditorAction::None
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.edit_buffer.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.edit_buffer.len() {
                        self.edit_buffer.remove(self.cursor_pos);
                    }
                    EditorAction::None
                }
                KeyCode::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.edit_buffer.len() {
                        self.cursor_pos += 1;
                    }
                    EditorAction::None
                }
                _ => EditorAction::None,
            }
        } else {
            match key {
                KeyCode::Up => {
                    if self.selected_field > 0 {
                        self.selected_field -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Down => {
                    if self.selected_field < self.fields.len().saturating_sub(1) {
                        self.selected_field += 1;
                    }
                    EditorAction::None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.start_editing();
                    EditorAction::None
                }
                KeyCode::Left | KeyCode::Right => {
                    // For enum fields, cycle through options
                    if let Some((_, _, value)) = self.fields.get_mut(self.selected_field) {
                        if let FieldValue::Enum { options, selected } = value {
                            if key == KeyCode::Right {
                                *selected = (*selected + 1) % options.len();
                            } else if *selected > 0 {
                                *selected -= 1;
                            } else {
                                *selected = options.len() - 1;
                            }
                            return EditorAction::ValueChanged;
                        } else if let FieldValue::Bool(b) = value {
                            *b = !*b;
                            return EditorAction::ValueChanged;
                        }
                    }
                    EditorAction::None
                }
                KeyCode::Esc => EditorAction::Close,
                KeyCode::Char('s') => EditorAction::Save,
                _ => EditorAction::None,
            }
        }
    }

    fn start_editing(&mut self) {
        if let Some((_, _, value)) = self.fields.get(self.selected_field) {
            match value {
                FieldValue::Text(_) | FieldValue::Number(_) => {
                    self.edit_buffer = value.as_display_string();
                    self.cursor_pos = self.edit_buffer.len();
                    self.is_editing = true;
                }
                FieldValue::Bool(_) | FieldValue::Enum { .. } => {
                    // These are toggled with arrow keys, not edited
                }
            }
        }
    }

    fn apply_edit(&mut self) {
        if let Some((_, _, value)) = self.fields.get_mut(self.selected_field) {
            match value {
                FieldValue::Text(_) => {
                    *value = FieldValue::Text(self.edit_buffer.clone());
                }
                FieldValue::Number(_) => {
                    if let Ok(num) = self.edit_buffer.parse::<f64>() {
                        *value = FieldValue::Number(num);
                    }
                }
                _ => {}
            }
        }
        self.edit_buffer.clear();
    }

    pub fn to_bindings(&self) -> HashMap<String, Value> {
        let mut bindings = HashMap::new();

        for (label, field, value) in &self.fields {
            let key = match field {
                EditorField::DeviceId => "device_id",
                EditorField::DisplayName => "display_name",
                EditorField::UnitLabel => "unit_of_measure_label",
                EditorField::MaxValue => "max_value",
                EditorField::WarnThreshold => "warn_threshold",
                EditorField::DangerThreshold => "danger_threshold",
                EditorField::ChartType => "chart_type",
                EditorField::IsBlinkable => "is_blinkable",
                EditorField::Label => "label",
                EditorField::ToggleOnLabel => "toggle_on_label",
                EditorField::ToggleOffLabel => "toggle_off_label",
                EditorField::Description => "description",
                EditorField::Model => "model",
                EditorField::CustomBinding { key } => {
                    // Skip module_type as it goes in base config
                    if key == "module_type" {
                        continue;
                    }
                    key.as_str()
                }
                _ => continue,
            };

            bindings.insert(key.to_string(), value.to_json());
        }

        bindings
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let title = if self.is_new_module {
            if self.fields.len() == 2 {
                " Create New Module - Select Type "
            } else {
                " Create New Module - Configure Fields "
            }
        } else {
            " Edit Module Configuration "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);

        let inner = block.inner(area);
        block.render(area, buf);

        // Split into two columns if space permits
        let (left_area, right_area) = if inner.width > 80 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(inner);
            (chunks[0], chunks[1])
        } else {
            (inner, Rect::default())
        };

        // Render fields
        let mut y = left_area.y;
        let max_y = left_area.bottom();

        for (idx, (label, _field, value)) in self.fields.iter().enumerate() {
            if y >= max_y {
                break;
            }

            let is_selected = idx == self.selected_field;
            let is_editing_this = is_selected && self.is_editing;

            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Render label
            let label_text = format!("{:20}", label);
            buf.set_string(left_area.x, y, &label_text, style);

            // Render value
            let value_x = left_area.x + 22;
            let value_width = left_area.width.saturating_sub(22);

            if is_editing_this {
                // Show edit buffer with cursor
                let display = if self.cursor_pos < self.edit_buffer.len() {
                    format!("{}│{}",
                            &self.edit_buffer[..self.cursor_pos],
                            &self.edit_buffer[self.cursor_pos..])
                } else {
                    format!("{}│", self.edit_buffer)
                };
                buf.set_string(value_x, y, &display, Style::default().fg(Color::Green));
            } else {
                let display = match value {
                    FieldValue::Bool(b) => {
                        if *b { "[X] true" } else { "[ ] false" }
                    }
                    FieldValue::Enum { options, selected } => {
                        &format!("< {} >", options.get(*selected).unwrap_or(&String::new()))
                    }
                    _ => &value.as_display_string(),
                };

                buf.set_string(value_x, y, display, style);
            }

            y += 1;
        }

        // Help text
        if right_area.width > 0 {
            let help_text = if self.is_new_module && self.fields.len() == 2 {
                vec![
                    "Create New Module:",
                    "",
                    "1. Enter module name",
                    "2. Select module type",
                    "3. Press Enter to continue",
                    "",
                    "↑/↓     - Navigate",
                    "←/→     - Change type",
                    "Enter   - Edit/Continue",
                    "Esc     - Cancel",
                ]
            } else if self.is_editing {
                vec![
                    "Editing mode:",
                    "",
                    "Enter - Save",
                    "Esc   - Cancel",
                    "←/→   - Move cursor",
                ]
            } else {
                vec![
                    "Navigation:",
                    "",
                    "↑/↓     - Select field",
                    "Enter   - Edit text",
                    "Space   - Edit text",
                    "←/→     - Toggle bool/enum",
                    "s       - Save config",
                    "Esc     - Close editor",
                ]
            };

            let mut help_y = right_area.y;
            for line in help_text {
                buf.set_string(
                    right_area.x + 2,
                    help_y,
                    line,
                    Style::default().fg(Color::DarkGray),
                );
                help_y += 1;
            }
        } else {
            // Show abbreviated help at bottom
            let help = if self.is_new_module && self.fields.len() == 2 {
                "←/→: Change Type | Esc: Cancel"
            } else if self.is_editing {
                "Enter: Save | Esc: Cancel"
            } else {
                " ←/→: Toggle | s: Save | Esc: Close"
            };

            buf.set_string(
                area.x + 2,
                area.bottom().saturating_sub(1),
                help,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EditorAction {
    None,
    ValueChanged,
    Save,
    Close,
    ModuleTypeSelected, // New action for when module type is picked
}
