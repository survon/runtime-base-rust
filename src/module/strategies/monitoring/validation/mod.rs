use crate::module::{
    ValidationError,
    ConfigValidator,
    MonitoringConfig,
};

impl ConfigValidator {
    pub fn validate_monitoring(cfg: &MonitoringConfig) -> color_eyre::Result<()> {
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
}
