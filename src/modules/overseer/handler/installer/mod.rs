mod install_module;
mod install_from_registry;
mod install_from_file;
mod list_registry_modules;
mod create_module_config;
mod copy_dir_recursive;

use std::path::{PathBuf};

pub struct ModuleInstaller {
    pub wasteland_path: PathBuf,
    pub archive_path: PathBuf,
    pub registry_url: String,
}
