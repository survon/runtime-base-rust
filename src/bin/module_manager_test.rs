use tempfile::tempdir;
use std::io::Write;

use survon_base_rust::module_manager::ModuleManager;
use survon_base_rust::orchestrator::Orchestrator;
use survon_base_rust::event_bus::EventBus;

use serde_json::json;

fn main() {
    // Set up temporary module directory
    let temp_dir = tempdir().unwrap();
    let modules_dir = temp_dir.path().to_path_buf();
    println!("Temporary modules directory: {:?}", modules_dir);

    // Create a mock orchestrator
    let event_bus = std::sync::Arc::new(EventBus::new());
    let orchestrator = std::sync::Arc::new(Orchestrator::new(event_bus));

    // Initialize the module manager
    let module_manager = ModuleManager::new(&modules_dir, orchestrator.clone());

    // Simulate a module zip file
    let module_zip = create_mock_module_zip();

    // Test module installation
    println!("Installing module...");
    let module_info = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(module_manager.install_module(&module_zip))
        .unwrap();
    println!("Module installed: {:?}", module_info);

    // Verify files are in the modules directory
    let installed_path = modules_dir.join(&module_info.name);
    println!("Installed module directory: {:?}", installed_path);
    assert!(installed_path.exists());

    // Test module uninstallation
    println!("Uninstalling module...");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(module_manager.uninstall_module(&module_info.name))
        .unwrap();
    println!("Module uninstalled.");

    // Verify module directory is removed
    assert!(!installed_path.exists());
    println!("All tests passed!");
}

// Helper function to create a mock module zip file
fn create_mock_module_zip() -> Vec<u8> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Add meta.json to the zip
        let meta = json!({
            "name": "test_module",
            "version": "0.1.0",
            "survon_runtime": "runtime-base-rust",
            "survon_runtime_version": ">=0.0.1",
            "description": "A test module for manual testing",
            "author": "Test Author",
            "dependencies": [],
            "supported_events": ["ping", "shutdown"]
        });

        zip.start_file("meta.json", options).unwrap();
        zip.write_all(serde_json::to_string_pretty(&meta).unwrap().as_bytes())
            .unwrap();

        // Add an implementation file
        zip.start_file("implementation.rs", options).unwrap();
        zip.write_all(b"// Implementation file for test module").unwrap();

        // Finish the zip
        zip.finish().unwrap();
    } // `zip` is dropped here, releasing the borrow on `cursor`

    cursor.into_inner()
}

