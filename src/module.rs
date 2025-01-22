use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{Result, SurvonError};
use serde_json::{Value, json};

#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;

    async fn handle_request(&self, request: Value) -> Result<Value>;

    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub supported_events: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct TestModule {
    pub name: String,
}

impl TestModule {
    /// Creates a new TestModule instance.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl Module for TestModule {
    fn name(&self) -> &str {
        "test_module"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Test module for unit tests"
    }

    async fn handle_request(&self, request: Value) -> Result<Value> {
        match request.get("command").and_then(Value::as_str) {
            Some("ping") => Ok(json!({"response": "pong"})),
            _ => Err(SurvonError::ModuleError("Invalid request".to_string())),
        }
    }

    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_module_handle_request() {
    let module = Box::new(TestModule { name: "test_module".to_string() });
    let result = module.handle_request(json!({
        "command": "ping"
    })).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({ "response": "pong" }));
}
