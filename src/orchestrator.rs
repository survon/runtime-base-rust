use std::sync::Arc;
use dashmap::DashMap;
use crate::error::{Result, SurvonError};
use crate::module::{Module, TestModule};
use crate::event_bus::{Event, EventBus};
use serde_json::{Value, json};

pub struct Orchestrator {
    modules: Arc<DashMap<String, Box<dyn Module>>>,
    event_bus: Arc<EventBus>,
}

impl Orchestrator {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            modules: Arc::new(DashMap::new()),
            event_bus,
        }
    }

    pub(crate) fn register_module(&self, module: Box<dyn Module>) -> Result<()> {
        let name = module.name().to_string();

        if self.modules.contains_key(&name) {
            println!("Module {} already registered. Replacing it.", name);
            self.modules.remove(&name);
        }

        self.modules.insert(name.clone(), module);
        println!("Module {} registered successfully.", name);
        Ok(())
    }

    pub fn unregister_module(&self, module_name: &str) -> Result<()> {
        println!(
            "Modules before unregistration: {:?}",
            self.modules.iter().map(|m| m.key().clone()).collect::<Vec<_>>()
        );
        if self.modules.remove(module_name).is_some() {
            println!("Module {} unregistered successfully.", module_name);
            println!(
                "Modules after unregistration: {:?}",
                self.modules.iter().map(|m| m.key().clone()).collect::<Vec<_>>()
            );
            Ok(())
        } else {
            println!("Module {} was not found during unregistration.", module_name);
            Err(SurvonError::ModuleError(format!(
                "Module {} not found in orchestrator",
                module_name
            )))
        }
    }

    pub async fn dispatch_request(&self, module_name: &str, request: Value) -> Result<Value> {
        println!("Dispatching request to module: {}", module_name);
        if let Some(module) = self.modules.get(module_name) {
            println!("Found module {}. Handling request.", module_name);
            module.handle_request(request).await
        } else {
            println!("Module {} not found.", module_name);
            Err(SurvonError::ModuleError(format!(
                "Module {} not found",
                module_name
            )))
        }
    }


    pub async fn broadcast_event(&self, event: Event) {
        self.event_bus.publish(&event);
    }
}


#[tokio::test]
async fn test_orchestrator_register_and_dispatch() {
    let event_bus = Arc::new(EventBus::new());
    let orchestrator = Orchestrator::new(event_bus);
    let module = Box::new(TestModule { name: "test_module".to_string() });

    orchestrator.register_module(module).unwrap();

    let response = orchestrator
        .dispatch_request("test_module", json!({ "command": "ping" }))
        .await
        .unwrap();

    assert_eq!(response, json!({ "response": "pong" }));
}

#[tokio::test]
async fn test_orchestrator_invalid_module() {
    let event_bus = Arc::new(EventBus::new());
    let orchestrator = Orchestrator::new(event_bus);

    let result = orchestrator
        .dispatch_request("nonexistent", json!({ "command": "ping" }))
        .await;

    assert!(result.is_err());
}

