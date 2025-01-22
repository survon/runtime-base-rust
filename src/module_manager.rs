use std::path::{Path, PathBuf};
use zip::ZipArchive;
use std::fs;
use std::io::Cursor;
use crate::error::{Result, SurvonError};
use crate::module::{ModuleInfo, TestModule};
use std::sync::Arc;
use crate::orchestrator::Orchestrator;

pub struct ModuleManager {
    modules_dir: PathBuf,
    orchestrator: Arc<Orchestrator>,
}

impl ModuleManager {
    pub fn new<P: AsRef<Path>>(modules_dir: P, orchestrator: Arc<Orchestrator>) -> Self {
        Self {
            modules_dir: modules_dir.as_ref().to_path_buf(),
            orchestrator
        }
    }

    pub async fn install_module(&self, zip_data: &[u8]) -> Result<ModuleInfo> {
        use std::fs;

        // Create a cursor for the ZIP data
        let cursor = std::io::Cursor::new(zip_data);

        // Create a ZIP archive from the cursor
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| SurvonError::ModuleError(format!("Failed to read ZIP archive: {}", e)))?;

        // Create a temporary directory for extraction
        let temp_dir = tempfile::tempdir()
            .map_err(|e| SurvonError::ModuleError(format!("Failed to create temp directory: {}", e)))?;

        // Extract all files into the temp directory
        archive
            .extract(&temp_dir)
            .map_err(|e| SurvonError::ModuleError(format!("Failed to extract ZIP archive: {}", e)))?;

        // Validate the presence of meta.json
        let meta_path = temp_dir.path().join("meta.json");
        if !meta_path.exists() {
            return Err(SurvonError::ModuleError("meta.json not found in the module".into()));
        }

        // Read and validate metadata
        let meta: ModuleInfo = serde_json::from_str(&fs::read_to_string(&meta_path)?)
            .map_err(|e| SurvonError::ModuleError(format!("Invalid meta.json format: {}", e)))?;

        // Create the module directory
        let module_dir = Path::new(&self.modules_dir).join(&meta.name);

        if module_dir.exists() {
            fs::remove_dir_all(&module_dir)
                .map_err(|e| SurvonError::ModuleError(format!("Failed to remove existing module directory: {}", e)))?;
        }
        fs::create_dir_all(&module_dir)
            .map_err(|e| SurvonError::ModuleError(format!("Failed to create module directory: {}", e)))?;

        // Move files from the temp directory to the module directory
        for entry in fs::read_dir(temp_dir.path())
            .map_err(|e| SurvonError::ModuleError(format!("Failed to read temp directory: {}", e)))?
        {
            let entry = entry.map_err(|e| SurvonError::ModuleError(format!("Failed to read entry: {}", e)))?;
            let file_name = entry.file_name();
            let source_path = entry.path();
            let destination_path = module_dir.join(&file_name);

            if file_name != "meta.json" {
                fs::copy(&source_path, &destination_path).map_err(|e| {
                    SurvonError::ModuleError(format!(
                        "Failed to copy file {:?} to {:?}: {}",
                        source_path, destination_path, e
                    ))
                })?;
            } else {
                fs::copy(&meta_path, &destination_path).map_err(|e| {
                    SurvonError::ModuleError(format!(
                        "Failed to copy meta.json to module directory: {}",
                        e
                    ))
                })?;
            }
        }

        // Register the module with the orchestrator
        let module = Box::new(TestModule { name: meta.name.clone() }); // Replace TestModule with actual module implementation
        self.orchestrator.register_module(module)?;

        Ok(meta)
    }


    pub async fn uninstall_module(&self, module_name: &str) -> Result<()> {
        // Ensure the module exists on disk
        let module_dir = self.modules_dir.join(module_name);
        if !module_dir.exists() {
            return Err(SurvonError::ModuleError(format!("Module {} not found", module_name)));
        }

        // Unregister the module from the orchestrator
        self.orchestrator.unregister_module(module_name)?;

        // Remove the module directory from disk
        fs::remove_dir_all(module_dir)?;
        Ok(())
    }

}
