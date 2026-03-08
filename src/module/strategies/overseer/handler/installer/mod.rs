mod copy_dir_recursive;
mod create_module_config;
mod install_from_file;
mod install_from_registry;
mod install_module;
mod list_registry_modules;

use std::path::PathBuf;

pub struct ModuleInstaller {
    pub wasteland_path: PathBuf,
    pub archive_path: PathBuf,
    pub registry_url: String,
}
