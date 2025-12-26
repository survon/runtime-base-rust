use std::collections::HashMap;

use crate::module::{ConfigValidator, ValidationError};

impl ConfigValidator {
    /// Quick validation for template compatibility
    pub fn validate_template_bindings(
        template: &str,
        bindings: &HashMap<String, serde_json::Value>,
    ) -> color_eyre::Result<()> {
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
