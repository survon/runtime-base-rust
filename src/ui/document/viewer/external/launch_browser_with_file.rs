use super::ExternalViewer;

impl ExternalViewer {
    pub(super)async fn launch_browser_with_file(&self, file_path: &str) -> color_eyre::Result<()> {
        let path = std::path::Path::new(file_path);
        self.launch_browser(path).await
    }
}
