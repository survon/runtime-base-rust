use std::error::Error;

mod module_manager;
mod ui;

use std::fs;
use std::path::Path;

fn ensure_module_directory() {
    let path = Path::new("/tmp/wasteland");
    if !path.exists() {
        if let Err(e) = fs::create_dir_all(path) {
            eprintln!("Failed to create {}: {}", path.display(), e);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    ensure_module_directory();

    // Scan /tmp/wasteland for ZIP files containing modules.
    let loaded_modules = module_manager::load_modules("/tmp/wasteland")?;

    if loaded_modules.is_empty() {
        println!("Please provide modules to continue. Place module ZIP files in /tmp/wasteland and restart the application.");
        return Ok(());
    }

    // Launch the TUI application.
    ui::run_app(loaded_modules)?;

    Ok(())
}
