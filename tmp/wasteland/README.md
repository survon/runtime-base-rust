# Module Directory Instructions

This directory (`/tmp/wasteland`) is used by the Survon runtime to locate and install external modules. Each module should be packaged as a ZIP file containing the module's dynamic library and a manifest file.

## How to Package a Module

Each module package should follow this structure:

```
module-name.zip
├── manifest.json
└── libmodule_name.so
```

- **manifest.json:**  
  A JSON file that provides the module's metadata. It must include at least:
  ```json
  {
    "name": "Module Name",
    "lib_file": "libmodule_name.so"
  }
  ```
Replace `"Module Name"` and `"libmodule_name.so"` with your module's actual name and dynamic library filename.

- **libmodule_name.so:**  
  The compiled dynamic library for your module.

## How to Use This Directory

1. **Prepare Your Module Package:**  
   Create your module package (a ZIP file) as described above.

2. **Place the Module ZIP File Here:**  
   Copy or move your module ZIP file into this directory. For example, if your module is called `module-example.zip`, ensure it is located in `/tmp/wasteland/module-example.zip`.

3. **Restart the Survon Runtime:**  
   When the Survon runtime starts, it will automatically scan this directory for module ZIP files, extract them, and load the modules based on the information in the manifest.

## Testing Your Module

- **Verify Contents:**  
  Ensure the ZIP file is correctly structured by unzipping it manually:
  ```bash
  unzip -l /tmp/wasteland/your_module.zip
  ```
  You should see the `manifest.json` and the `.so` file at the root of the ZIP.

- **Log Output:**  
  When the runtime starts, it will log messages indicating which modules were found and installed. Check the runtime logs for any errors or status messages.

## Additional Notes

- This folder is persistent within the container based on the Docker image build. For development, you can update the contents of `tmp/wasteland` on your host, rebuild the image, and test new modules.
- In a production system, a file picker or disk reader interface might be provided to manage module installation dynamically. For now, manually placing the ZIP files here is sufficient.
