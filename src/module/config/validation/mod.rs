mod error;
mod config_validator;

use crate::module::config::*;

pub use config_validator::ConfigValidator;
pub use error::ValidationError;

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
