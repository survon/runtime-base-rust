use color_eyre::Result;
use std::process::{Command, Stdio};
use std::path::Path;
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub installed: bool,
    pub running: bool,
    pub available_models: Vec<String>,
    pub recommended_model: String,
}

pub struct OllamaManager {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaManager {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_status(&self) -> OllamaStatus {
        let installed = self.is_ollama_installed();
        let running = self.is_ollama_running().await;
        let available_models = if running {
            self.get_available_models().await.unwrap_or_default()
        } else {
            Vec::new()
        };

        OllamaStatus {
            installed,
            running,
            available_models,
            recommended_model: "llama3.2".to_string(),
        }
    }

    fn is_ollama_installed(&self) -> bool {
        // Check if ollama command exists
        Command::new("ollama")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn is_ollama_running(&self) -> bool {
        match self.client
            .get(&format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn install_ollama(&self) -> Result<bool> {
        println!("Installing Ollama...");

        // Detect OS and install accordingly
        let install_success = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            self.install_ollama_unix().await?
        } else if cfg!(target_os = "windows") {
            self.install_ollama_windows().await?
        } else {
            return Ok(false);
        };

        if install_success {
            println!("Ollama installed successfully!");
            // Give it a moment to be available
            sleep(Duration::from_secs(2)).await;
        }

        Ok(install_success)
    }

    async fn install_ollama_unix(&self) -> Result<bool> {
        let output = Command::new("curl")
            .args(["-fsSL", "https://ollama.ai/install.sh"])
            .stdout(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let script = String::from_utf8(output.stdout)?;
        let output = Command::new("sh")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        Ok(output.success())
    }

    async fn install_ollama_windows(&self) -> Result<bool> {
        // For Windows, we'd need to download and run the installer
        // This is more complex and might require admin privileges
        println!("Windows installation requires manual download from https://ollama.ai");
        println!("Please download and install Ollama, then restart Survon.");
        Ok(false)
    }

    pub async fn start_ollama(&self) -> Result<bool> {
        if !self.is_ollama_installed() {
            return Ok(false);
        }

        println!("Starting Ollama service...");

        // Try to start Ollama as a background service
        let _child = Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Wait a bit for the service to start
        for i in 0..10 {
            sleep(Duration::from_millis(500)).await;
            if self.is_ollama_running().await {
                println!("Ollama service started successfully!");
                return Ok(true);
            }
            if i == 4 {
                println!("Still starting Ollama service...");
            }
        }

        Ok(false)
    }

    pub async fn install_model(&self, model_name: &str) -> Result<bool> {
        if !self.is_ollama_running().await {
            return Ok(false);
        }

        println!("Installing model: {}", model_name);
        println!("This may take several minutes depending on model size...");

        let output = Command::new("ollama")
            .args(["pull", model_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if output.success() {
            println!("Model {} installed successfully!", model_name);
            Ok(true)
        } else {
            println!("Failed to install model: {}", model_name);
            Ok(false)
        }
    }

    pub async fn ensure_ollama_ready(&self, preferred_model: Option<&str>) -> Result<OllamaStatus> {
        let mut status = self.check_status().await;

        // Step 1: Install Ollama if not present
        if !status.installed {
            println!("Ollama not found. Installing...");
            if !self.install_ollama().await? {
                println!("Failed to install Ollama automatically.");
                println!("Please visit https://ollama.ai to install manually.");
                return Ok(status);
            }
            status.installed = true;
        }

        // Step 2: Start Ollama if not running
        if !status.running {
            println!("Starting Ollama service...");
            if !self.start_ollama().await? {
                println!("Failed to start Ollama service.");
                println!("Try running 'ollama serve' manually in another terminal.");
                return Ok(status);
            }
            status.running = true;
        }

        // Step 3: Install a model if none available
        status.available_models = self.get_available_models().await.unwrap_or_default();

        if status.available_models.is_empty() {
            let model_to_install = preferred_model.unwrap_or(&status.recommended_model);
            println!("No models found. Installing recommended model: {}", model_to_install);

            if self.install_model(model_to_install).await? {
                status.available_models.push(model_to_install.to_string());
            }
        }

        Ok(status)
    }

    pub fn get_install_instructions() -> String {
        if cfg!(target_os = "windows") {
            "Windows: Download installer from https://ollama.ai\nThen restart Survon.".to_string()
        } else {
            "Survon can auto-install Ollama for you.\nPress 'i' to install, or visit https://ollama.ai".to_string()
        }
    }
}
