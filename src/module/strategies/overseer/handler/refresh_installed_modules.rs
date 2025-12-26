use std::fs;
use super::OverseerHandler;

impl OverseerHandler {
    pub(in crate::module) fn refresh_installed_modules(&mut self) {
        self.installed_modules.clear();

        if let Ok(entries) = fs::read_dir(&self.wasteland_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if !name_str.starts_with('.') {
                                self.installed_modules.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
