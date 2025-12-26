use crate::module::{ComConfig, ConfigValidator, ValidationError};

impl ConfigValidator {
    pub fn validate_com(cfg: &ComConfig) -> color_eyre::Result<()> {
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
}
