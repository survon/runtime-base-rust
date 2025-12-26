use crate::module::strategies::overseer::handler::{
    RegistryManifest,
};

use super::ModuleInstaller;

impl ModuleInstaller {
    pub(in crate::module) async fn list_registry_manifests(&self) -> color_eyre::Result<Vec<RegistryManifest>> {
        // Mock - same as in handler for now
        // TODO wire up
        Ok(vec![
            RegistryManifest {
                id: "pressure_monitor".to_string(),
                name: "Pressure Monitor".to_string(),
                description: "Monitor hydraulic or pneumatic pressure".to_string(),
                version: "1.0.0".to_string(),
                author: "Survon Community".to_string(),
                module_type: "monitoring".to_string(),
                template: "gauge_card".to_string(),
                download_url: format!("{}/manifests/pressure_monitor/download", self.registry_url),
                checksum: "abc123".to_string(),
            },
            RegistryManifest {
                id: "temperature_sensor".to_string(),
                name: "Temperature Sensor".to_string(),
                description: "Monitor ambient or equipment temperature".to_string(),
                version: "1.2.0".to_string(),
                author: "Survon Core".to_string(),
                module_type: "monitoring".to_string(),
                template: "gauge_card".to_string(),
                download_url: format!("{}/manifests/temperature_sensor/download", self.registry_url),
                checksum: "def456".to_string(),
            },
            RegistryManifest {
                id: "gate_controller".to_string(),
                name: "Gate Controller".to_string(),
                description: "Remote gate/door control".to_string(),
                version: "2.0.0".to_string(),
                author: "Survon Core".to_string(),
                module_type: "com".to_string(),
                template: "toggle_switch".to_string(),
                download_url: format!("{}/manifests/gate_controller/download", self.registry_url),
                checksum: "ghi789".to_string(),
            },
        ])
    }
}
