use std::fs;

use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn restore_module(&self, archive_name: &str, new_name: Option<String>) -> color_eyre::Result<()> {
        let archive_source = self.archive_path.join(archive_name);

        if !archive_source.exists() {
            return Err(color_eyre::eyre::eyre!("Archived module not found"));
        }

        // Extract original name (remove timestamp suffix)
        let original_name = archive_name.split('_').next().unwrap_or(archive_name);

        let restore_name = new_name.unwrap_or_else(|| original_name.to_string());
        let restore_dest = self.wasteland_path.join(&restore_name);

        if restore_dest.exists() {
            return Err(color_eyre::eyre::eyre!("Module already exists"));
        }

        // Move back from archive
        fs::rename(&archive_source, &restore_dest)?;

        Ok(())
    }
}
