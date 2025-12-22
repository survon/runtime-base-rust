use std::fs;
use crate::modules::wasteland_manager::handler::WastelandManagerHandler;

impl WastelandManagerHandler {
    pub(super) fn archive_module(&self, module_name: &str) -> color_eyre::Result<()> {
        let module_path = self.wasteland_path.join(module_name);

        if !module_path.exists() {
            return Err(color_eyre::eyre::eyre!("Module not found"));
        }

        // Create archive directory if needed
        fs::create_dir_all(&self.archive_path)?;

        // Generate unique archive name with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let archive_name = format!("{}_{}", module_name, timestamp);
        let archive_dest = self.archive_path.join(&archive_name);

        // Move to archive
        fs::rename(&module_path, &archive_dest)?;

        Ok(())
    }
}
