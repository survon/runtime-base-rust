use std::fs;
use std::path::Path;

use super::ModuleInstaller;

impl ModuleInstaller {
    // TODO move to util
    pub(super) fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> color_eyre::Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }
}
