use std::path::Path;
use tokio::io::AsyncBufReadExt;

use super::DocumentManager;

impl DocumentManager {
    pub fn open_document(&mut self, file_path: String) {
        let (actual_path, _page_number) = if file_path.contains("#page=") {
            let parts: Vec<&str> = file_path.split("#page=").collect();
            let page = parts.get(1).and_then(|p| p.parse::<u32>().ok());
            (parts[0].to_string(), page)
        } else {
            (file_path.clone(), None)
        };

        let path = Path::new(&actual_path);
        let content = self.viewer.get_direct_view_content(path)
            .or_else(|| {
                self.viewer.view_document(path).ok()
            });

        if let Some(content) = content {
            if let Some(external_viewer) = &self.external_viewer {
                let viewer = external_viewer.clone();
                let path_clone = file_path.clone();

                tokio::spawn(async move {
                    if let Err(e) = viewer.show_document_external(&path_clone, &content).await {
                        panic!("Failed to launch external viewer: {}", e);
                    }
                });
            }
        }
    }
}
