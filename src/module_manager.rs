use libloading::{Library, Symbol};
use serde::Deserialize;
use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

use survon_module_api::{Module, ModuleCreateFunc};

/// The structure of the manifest file expected inside each module ZIP.
#[derive(Deserialize)]
struct ModuleManifest {
    /// The display name (and namespace) of the module.
    name: String,
    /// The filename of the dynamic library (e.g. "module.so")
    lib_file: String,
}

/// A loaded module instance along with its library handle (to keep it alive).
pub struct LoadedModule {
    pub module: Box<dyn Module>,
    // Retain the library handle to ensure the code remains loaded.
    _lib: Library,
}

/// Scans the given directory for .zip files, extracts them,
/// reads each manifest, loads the dynamic library, and creates a module instance.
pub fn load_modules(dir: &str) -> Result<Vec<LoadedModule>, Box<dyn Error>> {
    let mut loaded_modules = Vec::new();

    if !Path::new(dir).exists() {
        println!("Module directory '{}' does not exist.", dir);
        return Ok(loaded_modules);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            println!("Found module ZIP: {:?}", path);

            // Create a directory for extraction (e.g. remove the .zip extension)
            let extract_dir = path.with_extension("");
            fs::create_dir_all(&extract_dir)?;

            // Open the ZIP file and extract its contents.
            let file = File::open(&path)?;
            let mut archive = ZipArchive::new(file)?;
            archive.extract(&extract_dir)?;

            // Read the manifest.json file from the extraction directory.
            let manifest_path = extract_dir.join("manifest.json");
            if !manifest_path.exists() {
                eprintln!("manifest.json not found in {:?}", extract_dir);
                continue;
            }
            let manifest_file = File::open(&manifest_path)?;
            let manifest: ModuleManifest = serde_json::from_reader(manifest_file)?;

            // Determine the full path to the dynamic library.
            let lib_path = extract_dir.join(&manifest.lib_file);
            if !lib_path.exists() {
                eprintln!("Library file {:?} not found.", lib_path);
                continue;
            }

            unsafe {
                // Load the dynamic library.
                let lib = Library::new(lib_path)?;
                // Look up the exported symbol "create_module".
                let func: Symbol<ModuleCreateFunc> = lib.get(b"create_module")?;
                // Convert the module name to a C string.
                let c_name = CString::new(manifest.name.clone())?;
                // Call the module's constructor.
                let module_instance = func(c_name.as_ptr());
                loaded_modules.push(LoadedModule {
                    module: module_instance,
                    _lib: lib, // keep the library loaded.
                });
            }
        }
    }

    Ok(loaded_modules)
}
