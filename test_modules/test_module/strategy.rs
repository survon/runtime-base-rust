pub struct TestModule;

#[async_trait]
impl Module for TestModule {
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
