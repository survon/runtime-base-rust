// src/util/service/discovery.rs
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilService {
    pub hostname: String,
    pub name: String,
    pub title: String,
    pub position: String,
    pub endpoint: String,
    pub model: String,
    pub status: String, // "available", "out_of_office", "error"
}

#[derive(Debug, Clone, Deserialize)]
struct ServiceMetadata {
    service_type: String,
    council_position: String,
    position_display: String,
    model_name: String,
    api_contract: String,
    endpoints: ServiceEndpoints,
}

#[derive(Debug, Clone, Deserialize)]
struct ServiceEndpoints {
    ollama_api: String,
}

pub struct ServiceDiscovery {
    scan_pattern: String,
    web_port: u16,
}

impl ServiceDiscovery {
    pub fn new(scan_pattern: String, web_port: u16) -> Self {
        Self { scan_pattern, web_port }
    }

    /// Discover council services on the network
    pub async fn discover_services(&self) -> Result<Vec<CouncilService>> {
        let mut services = Vec::new();

        // Try common hostnames based on pattern
        let positions = vec![
            "physician", "agronomist", "botanist", "rancher",
            "security", "engineer", "crafter", "technologist",
            "counselor", "wilderness", "storyteller"
        ];

        for position in positions {
            let hostname = format!("survon-{}.local", position);
            if let Ok(service) = self.check_service(&hostname).await {
                services.push(service);
            }
        }

        Ok(services)
    }

    /// Check if a specific hostname has a valid service
    async fn check_service(&self, hostname: &str) -> Result<CouncilService> {
        let url = format!("http://{}:{}/.survon/service.json", hostname, self.web_port);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()?;

        let response = client.get(&url).send().await?;
        let metadata: ServiceMetadata = response.json().await?;

        // Validate contract
        if metadata.api_contract != "survon-llm-service-v1" {
            return Err(color_eyre::eyre::eyre!("Invalid API contract"));
        }

        // Test if Ollama API is responding
        let status = match self.test_ollama_endpoint(&metadata.endpoints.ollama_api).await {
            Ok(_) => "available",
            Err(_) => "out_of_office",
        };

        Ok(CouncilService {
            hostname: hostname.to_string(),
            name: metadata.position_display.clone(),
            title: metadata.position_display,
            position: metadata.council_position,
            endpoint: metadata.endpoints.ollama_api,
            model: metadata.model_name,
            status: status.to_string(),
        })
    }

    /// Test if Ollama endpoint is responding
    async fn test_ollama_endpoint(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}/api/tags", endpoint);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        client.get(&url).send().await?;
        Ok(())
    }
}
