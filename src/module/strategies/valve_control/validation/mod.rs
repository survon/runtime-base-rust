use crate::module::{ConfigValidator, ValidationError, ValveControlConfig};

impl ConfigValidator {
    pub fn validate_valve_control(cfg: &ValveControlConfig) -> color_eyre::Result<()> {
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
}
