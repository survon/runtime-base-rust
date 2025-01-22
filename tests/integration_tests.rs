use survon_base_rust::{Runtime, RuntimeConfig}; // Adjust your imports as needed
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use survon_base_rust::utility::compute_md5;
use serde_json::{json};

#[tokio::test]
async fn test_dynamic_module_lifecycle() {
    // Create a temporary directory for the runtime's modules_dir
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().to_path_buf();
    let config = RuntimeConfig {
        modules_dir: modules_dir.clone().to_string_lossy().into_owned(),
        max_concurrent_tasks: 10,
    };

    // Initialize the runtime
    let runtime = Runtime::new(config).await.unwrap();

    // Create a dummy module in the modules_dir
    let module_dest_path = modules_dir.join("test_module");
    fs::create_dir_all(&module_dest_path).unwrap();
    fs::write(
        module_dest_path.join("meta.json"),
        serde_json::to_string_pretty(&json!({
            "name": "test_module",
            "version": "0.1.0",
            "survon_runtime": "runtime-base-rust",
            "survon_runtime_version": ">=0.0.1",
            "author": "Test Author",
            "description": "A test module for integration testing",
            "supported_events": ["ping", "shutdown"],
            "dependencies": []
        }))
            .unwrap(),
    )
        .unwrap();

    // Load the module
    runtime.load_modules().await.unwrap();

    // Dispatch a request to the module
    let response = runtime
        .dispatch_request("test_module", json!({ "command": "ping" }))
        .await
        .unwrap();
    assert_eq!(response, json!({ "response": "pong" }));

    // Uninstall the module
    runtime.uninstall_module("test_module").await.unwrap();

    // Ensure the module is no longer dispatchable
    let result = runtime
        .dispatch_request("test_module", json!({ "command": "ping" }))
        .await;
    assert!(result.is_err(), "Module should not respond after uninstallation");

    // Verify the module directory is removed
    assert!(
        !module_dest_path.exists(),
        "Module directory should be removed after uninstallation"
    );
}
