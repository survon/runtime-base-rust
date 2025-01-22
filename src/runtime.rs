use std::path::Path;
use std::sync::Arc;
use std::io::Write;
use serde_json::{Value,json};
use zip::write::FileOptions;
use tempfile::TempDir;
use crate::error::{Result, SurvonError};
use crate::event_bus::EventBus;
use crate::module::{Module, ModuleInfo, TestModule};
use crate::module_manager::ModuleManager;
use crate::orchestrator::Orchestrator;

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub modules_dir: String,
    pub max_concurrent_tasks: usize,
}

pub struct Runtime {
    modules: dashmap::DashMap<String, Box<dyn Module>>,
    event_bus: Arc<EventBus>,
    config: RuntimeConfig,
    module_manager: ModuleManager,
    orchestrator: Arc<Orchestrator>,
}

impl Runtime {
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        let event_bus = Arc::new(EventBus::new());
        let orchestrator = Arc::new(Orchestrator::new(event_bus.clone()));
        let module_manager = ModuleManager::new(&config.modules_dir, orchestrator.clone());

        Ok(Self {
            modules: dashmap::DashMap::new(),
            event_bus: event_bus.clone(),
            config,
            module_manager,
            orchestrator,
        })
    }

    pub async fn load_modules(&self) -> Result<()> {
        let modules_dir = Path::new(&self.config.modules_dir);
        if !modules_dir.exists() {
            std::fs::create_dir_all(modules_dir)?;
            println!("Created modules directory: {:?}", modules_dir);
        }

        for entry in std::fs::read_dir(modules_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                println!("Attempting to load module at path: {:?}", entry.path());
                self.load_module(&entry.path()).await?;
                println!("Successfully loaded module: {:?}", entry.path());
            }
        }
        Ok(())
    }

    async fn load_module(&self, path: &Path) -> Result<()> {
        let meta_path = path.join("meta.json");
        if !meta_path.exists() {
            return Err(SurvonError::ModuleError("meta.json not found".into()));
        }

        let meta: ModuleInfo = serde_json::from_str(&std::fs::read_to_string(meta_path)?)?;
        println!("Loaded module metadata: {:?}", meta);

        // Create a dummy module based on meta (replace this with actual module instantiation logic)
        let module = Box::new(TestModule::new(meta.name.clone())) as Box<dyn Module>;

        // Register the module with the orchestrator
        self.orchestrator.register_module(module)?;
        println!("Module {} registered successfully with the orchestrator.", meta.name);

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        // Initialize all modules
        for module in self.modules.iter() {
            module.initialize().await?;
        }

        // Main event loop
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        // Shutdown all modules
        for module in self.modules.iter() {
            module.shutdown().await?;
        }

        Ok(())
    }

    // Add these methods to Runtime impl:
    pub async fn install_module(&self, zip_data: &[u8]) -> Result<ModuleInfo> {
        self.module_manager.install_module(zip_data).await
    }

    pub async fn uninstall_module(&self, module_name: &str) -> Result<()> {
        self.module_manager.uninstall_module(module_name).await
    }

    pub async fn dispatch_request(&self, module_name: &str, request: Value) -> Result<Value> {
        println!("Dispatching request to module: {}", module_name);
        let response = self.orchestrator.dispatch_request(module_name, request).await;
        match &response {
            Ok(value) => println!("Request successful: {:?}", value),
            Err(err) => println!("Request failed: {:?}", err),
        }
        response
    }
}


#[tokio::test]
async fn test_runtime_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = RuntimeConfig {
        modules_dir: temp_dir.path().to_string_lossy().into_owned(),
        max_concurrent_tasks: 10,
    };

    let runtime = Runtime::new(config).await;
    assert!(runtime.is_ok());
}

#[tokio::test]
async fn test_runtime_module_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config = RuntimeConfig {
        modules_dir: temp_dir.path().to_string_lossy().into_owned(),
        max_concurrent_tasks: 10,
    };

    let runtime = Runtime::new(config).await.unwrap();
    let result = runtime.load_modules().await;
    assert!(result.is_ok());
}

// Integration test example
#[tokio::test]
async fn test_full_module_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let config = RuntimeConfig {
        modules_dir: temp_dir.path().to_string_lossy().into_owned(),
        max_concurrent_tasks: 10,
    };

    println!("Temporary directory created at: {:?}", temp_dir.path());

    let runtime = Runtime::new(config).await.unwrap();

    // Create a test module zip
    let zip_data = create_test_module_zip().await.unwrap();
    println!("Test module zip created.");

    // Install module
    let module_info = runtime.install_module(&zip_data).await.unwrap();
    println!("Module installed: {:?}", module_info);
    assert_eq!(module_info.name, "test_module");

    // Verify module directory content
    println!(
        "Module directory contents: {:?}",
        std::fs::read_dir(temp_dir.path()).unwrap().collect::<Vec<_>>()
    );

    // Load modules
    runtime.load_modules().await.unwrap();
    println!("Modules loaded.");

    // Test module functionality
    let response = runtime
        .dispatch_request("test_module", json!({ "command": "ping" }))
        .await
        .unwrap();
    println!("Response from module: {:?}", response);
    assert_eq!(response, json!({ "response": "pong" }));

    // Uninstall module
    runtime.uninstall_module("test_module").await.unwrap();
    println!("Module uninstalled successfully.");
}

// Helper function for creating test module zip
async fn create_test_module_zip() -> Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Create meta.json
        let meta = json!({
            "name": "test_module",
            "version": "0.1.0",
            "description": "A test module for integration testing",
            "author": "Test Author",
            "dependencies": [],
            "supported_events": ["ping", "shutdown"]
        });

        // Write meta.json to zip
        zip.start_file("meta.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&meta)?.as_bytes())?;

        // Add a mock strategy file
        zip.start_file("strategy.rs", options)?;
        let strategy = r#"
            pub struct TestModule;

            #[async_trait]
            impl Module for TestModule {
                async fn handle_request(&self, request: Value) -> Result<Value> {
                    match request.get("command").and_then(Value::as_str) {
                        Some("ping") => Ok(json!({"response": "pong"})),
                        _ => Err("Invalid request".into())
                    }
                }

                async fn initialize(&self) -> Result<()> {
                    Ok(())
                }

                async fn shutdown(&self) -> Result<()> {
                    Ok(())
                }
            }
        "#;
        zip.write_all(strategy.as_bytes())?;

        // Finish creating the zip
        zip.finish()?;
    } // `zip` is dropped here, releasing the borrow on `cursor`

    Ok(cursor.into_inner())
}


