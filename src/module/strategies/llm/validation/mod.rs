use crate::module::{ConfigValidator, LlmConfig, ValidationError};

impl ConfigValidator {
    pub fn validate_llm(cfg: &LlmConfig) -> color_eyre::Result<()> {
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
}
