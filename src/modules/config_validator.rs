// src/modules/config_validator.rs
// Validates module configs against their declared type schemas

use crate::modules::config_schema::*;
use color_eyre::Result;
use std::collections::HashMap;
use crate::{log_debug,log_error};

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub error: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

impl std::error::Error for ValidationError {}

pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a module config against its declared type
    pub fn validate(config_yaml: &str) -> Result<TypedModuleConfig> {
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

        // Now deserialize with proper type - with better error handling
        let typed_config: TypedModuleConfig = serde_yaml::from_str(config_yaml)
            .map_err(|e| {
                log_error!("Failed to deserialize as TypedModuleConfig: {}", e);
                log_error!("YAML was:\n{}", config_yaml);
                ValidationError {
                    field: "config".to_string(),
                    error: format!("Failed to deserialize as {}: {}", module_type, e),
                }
            })?;

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

    fn validate_monitoring(cfg: &MonitoringConfig) -> Result<()> {
        let b = &cfg.bindings;

        // Required fields
        if b.device_id.is_empty() {
            return Err(ValidationError {
                field: "bindings.device_id".to_string(),
                error: "Cannot be empty".to_string(),
            }.into());
        }

        if b.display_name.is_empty() {
            return Err(ValidationError {
                field: "bindings.display_name".to_string(),
                error: "Cannot be empty".to_string(),
            }.into());
        }

        // Validate thresholds if present
        if let (Some(max), Some(warn)) = (b.max_value, b.warn_threshold) {
            if warn > max {
                return Err(ValidationError {
                    field: "bindings.warn_threshold".to_string(),
                    error: format!("Cannot exceed max_value ({})", max),
                }.into());
            }
        }

        if let (Some(warn), Some(danger)) = (b.warn_threshold, b.danger_threshold) {
            if danger < warn {
                return Err(ValidationError {
                    field: "bindings.danger_threshold".to_string(),
                    error: format!("Should be >= warn_threshold ({})", warn),
                }.into());
            }
        }

        // Validate chart_type if present
        if let Some(chart_type) = &b.chart_type {
            let valid_types = ["line", "bar", "sparkline"];
            if !valid_types.contains(&chart_type.as_str()) {
                return Err(ValidationError {
                    field: "bindings.chart_type".to_string(),
                    error: format!("Must be one of: {:?}", valid_types),
                }.into());
            }
        }

        Ok(())
    }

    fn validate_valve_control(cfg: &ValveControlConfig) -> Result<()> {
        let b = &cfg.bindings;

        if b.device_id.is_empty() {
            return Err(ValidationError {
                field: "bindings.device_id".to_string(),
                error: "Cannot be empty".to_string(),
            }.into());
        }

        if b.label.is_empty() {
            return Err(ValidationError {
                field: "bindings.label".to_string(),
                error: "Cannot be empty".to_string(),
            }.into());
        }

        // Validate position is in range
        if b.b < 0 || b.b > 100 {
            return Err(ValidationError {
                field: "bindings.b".to_string(),
                error: "Position must be 0-100".to_string(),
            }.into());
        }

        Ok(())
    }

    fn validate_llm(cfg: &LlmConfig) -> Result<()> {
        let valid_models = ["search", "summarizer", "council"];
        if !valid_models.contains(&cfg.model.as_str()) {
            return Err(ValidationError {
                field: "model".to_string(),
                error: format!("Must be one of: {:?}", valid_models),
            }.into());
        }

        // If council model, service_discovery should be configured
        if cfg.model == "council" && cfg.service_discovery.is_none() {
            return Err(ValidationError {
                field: "service_discovery".to_string(),
                error: "Required for council model".to_string(),
            }.into());
        }

        Ok(())
    }

    fn validate_com(cfg: &ComConfig) -> Result<()> {
        // Com modules should have either toggle_switch or activity_card bindings
        let b = &cfg.bindings;

        let has_toggle = b.state.is_some() && b.label.is_some();
        let has_activity = b.activity_log.is_some();

        if !has_toggle && !has_activity {
            return Err(ValidationError {
                field: "bindings".to_string(),
                error: "Must have either toggle switch fields or activity_log".to_string(),
            }.into());
        }

        Ok(())
    }

    /// Quick validation for template compatibility
    pub fn validate_template_bindings(
        template: &str,
        bindings: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let required = match template {
            "gauge_card" => vec![
                "a", "device_id", "display_name", "unit_of_measure_label",
                "max_value", "warn_threshold", "danger_threshold",
            ],
            "chart_card" => vec![
                "a", "device_id", "display_name", "unit_of_measure_label",
                "chart_type", "max_value",
            ],
            "status_badge_card" => vec![
                "a", "device_id", "is_blinkable",
            ],
            "toggle_switch" => vec![
                "state", "label", "toggle_on_label", "toggle_off_label",
            ],
            "llm_card" => vec![
                "model_info", "chat_history", "chat_input",
            ],
            _ => vec![],
        };

        for field in required {
            if !bindings.contains_key(field) {
                return Err(ValidationError {
                    field: format!("bindings.{}", field),
                    error: format!("Required by template '{}'", template),
                }.into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_monitoring_config() {
        let yaml = r#"
name: "Test Sensor"
module_type: monitoring
bus_topic: "test"
template: "gauge_card"
bindings:
  a: 0.0
  b: 0.0
  c: 0.0
  device_id: "test01"
  device_type: "sensor"
  firmware_version: "1.0.0"
  display_name: "Test"
  unit_of_measure_label: "°C"
  max_value: 100.0
  warn_threshold: 60.0
  danger_threshold: 85.0
  is_blinkable: true
"#;

        let result = ConfigValidator::validate(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_thresholds() {
        let yaml = r#"
name: "Test Sensor"
module_type: monitoring
bus_topic: "test"
template: "gauge_card"
bindings:
  a: 0.0
  b: 0.0
  c: 0.0
  device_id: "test01"
  device_type: "sensor"
  firmware_version: "1.0.0"
  display_name: "Test"
  unit_of_measure_label: "°C"
  max_value: 100.0
  warn_threshold: 110.0
  danger_threshold: 85.0
"#;

        let result = ConfigValidator::validate(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_module_type() {
        let yaml = r#"
name: "Test"
module_type: invalid_type
bus_topic: "test"
template: ""
bindings: {}
"#;

        let result = ConfigValidator::validate(yaml);
        assert!(result.is_err());
    }
}
