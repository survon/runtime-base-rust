use super::ExternalViewer;

impl ExternalViewer {
    pub fn new() -> color_eyre::Result<Self> {
        let temp_dir = std::path::PathBuf::from("/tmp/survon_viewer");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self { temp_dir })
    }
}
