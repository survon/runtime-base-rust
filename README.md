# runtime-base-rust

This is a Ratatui-based terminal UI application for Survon, an offline modular survival system focused on resilience for off-grid scenarios.

## Installation
Clone and build manually, or use the Survon OS installer for RPi 3B (armhf):
```bash
curl -sSL https://raw.githubusercontent.com/survon/survon-os/master/scripts/install.sh | bash --cleanup
```
- Installer compiles llama-cli from llama.cpp, downloads models (interactive: phi3-mini or custom URL), builds release binary.
- Post-install: Reboot for bash menu; launch app via option 4 (takes over terminal like a full-screen React component).

## Usage
- Run: `./target/release/runtime-base-rust` (or `survon-runtime` post-install).
- Test LLM:
  ```bash
  ./bundled/llama-cli --model bundled/models/phi3-mini.gguf --ctx-size 512 --threads 4 --prompt "<|system|>You are Survon, a helpful homestead assistant.<|end|><|user|>do you have tacos?<|end|><|assistant|>" --n-predict 50 --simple-io
  ```
- In code: Load model via `env::var("MODEL_NAME").unwrap_or("phi3-mini.gguf".to_string())` (assumption disclosed: Based on history; verify in main.rs).
- Config: Edit via menu option 2 (sets ~/.bashrc; source for immediate use, like reloading a Node env file).

## License
Copyright (c) Sean Cannon

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>).
