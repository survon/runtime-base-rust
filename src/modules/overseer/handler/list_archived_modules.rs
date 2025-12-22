use std::fs;

use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn list_archived_modules(&self) -> color_eyre::Result<Vec<String>> {
        if !self.archive_path.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.archive_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        Ok(entries)
    }
}
