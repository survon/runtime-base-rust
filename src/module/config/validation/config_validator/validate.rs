use crate::{
    log_debug,
    module::{
        config::validation::{
            config_validator::ConfigValidator,
            error::ValidationError
        },
        get_supported_templates,
        TypedModuleConfig,
    }
};

impl ConfigValidator {
    /// Validate a module config against its declared type
    pub fn validate(config_yaml: &str) -> color_eyre::Result<TypedModuleConfig> {
        // First, deserialize as generic to get module_type
        let generic: serde_json::Value = serde_yaml::from_str(config_yaml)
            .map_err(|e| ValidationError {
                field: "yaml".to_string(),
                error: format!("Failed to parse YAML: {}", e),
            })?;

        log_debug!("Generic: {}", serde_json::to_string_pretty(&generic).unwrap_or_default());

        let module_type = generic.get("module_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ValidationError {
                field: "module_type".to_string(),
                error: "Missing required field".to_string(),
            })?;

        log_debug!("module_type: {}", module_type);

        // Validate template if present
        if let Some(template) = generic.get("template").and_then(|v| v.as_str()) {
            if !template.is_empty() && !get_supported_templates().contains(&template) {
                return Err(ValidationError {
                    field: "template".to_string(),
                    error: format!("Unsupported template: {}. Must be one of: {:?}",
                                   template, get_supported_templates()),
                }.into());
            }
        }

        // Now deserialize with proper type
        let typed_config: TypedModuleConfig = serde_yaml::from_str(config_yaml)?;

        // Type-specific validation
        match &typed_config {
            TypedModuleConfig::Monitoring(cfg) => {
                Self::validate_monitoring(cfg)?;
            }
            TypedModuleConfig::ValveControl(cfg) => {
                Self::validate_valve_control(cfg)?;
            }
            TypedModuleConfig::Llm(cfg) => {
                Self::validate_llm(cfg)?;
            }
            TypedModuleConfig::Com(cfg) => {
                Self::validate_com(cfg)?;
            }
            _ => {
                // Other types have minimal validation requirements
            }
        }

        Ok(typed_config)
    }
}
