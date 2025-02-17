# Survon Base Rust

Survon is an offline-first, modular TUI-based system that dynamically loads external modules (plugins) at runtime. This prototype demonstrates how to cross-compile the Survon runtime for ultra low-resource devices (e.g., Raspberry Pi) using a musl-linked ARM64 target, deploy it in a Docker container, and install modules via a designated module directory.

> **Note:**  
> This README assumes you are building on macOS. Similar steps apply for Linux.

---

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Environment Setup and Cross-Compilation](#environment-setup-and-cross-compilation)
  - [1. Install the Cross-Compiler](#1-install-the-cross-compiler)
  - [2. Configure Cargo](#2-configure-cargo)
  - [3. Build the Runtime](#3-build-the-runtime)
  - [4. Verify the Binary](#4-verify-the-binary)
- [Docker Deployment (Optional)](#docker-deployment-optional)
- [Module Installation and Setup](#module-installation-and-setup)
- [Troubleshooting](#troubleshooting)
- [License](#license)

---

## Overview

The Survon runtime is a TUI-based application that manages and displays external modules. Modules are delivered as ZIP files containing a manifest (with the module name and dynamic library filename) and the compiled dynamic library.  
For low-resource systems, we cross-compile the runtime for the `aarch64-unknown-linux-musl` target to produce a self-contained ELF binary that runs on musl-based systems (like Alpine Linux).

---

## Prerequisites

- **Rust & Cargo:** Install via [rustup](https://rustup.rs)
- **Homebrew (macOS):** Install via [brew.sh](https://brew.sh)
- **Docker:** For containerized deployment (optional)

---

## Environment Setup and Cross-Compilation

### 1. Install the Cross-Compiler

On macOS, install the ARM Linux musl cross-compiler using Homebrew:

```bash
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-musl
```

Verify the installation:

```bash
aarch64-linux-musl-gcc --version
```

### 2. Configure Cargo

In the root of your project (`runtime-base-rust`), create (or update) the `.cargo/config.toml` file with:

```toml
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
```

This instructs Cargo to use the cross-compiler when targeting musl.

### 3. Build the Runtime

First, add the musl target:

```bash
rustup target add aarch64-unknown-linux-musl
```

Then build the project in release mode:

```bash
cargo build --release --target=aarch64-unknown-linux-musl
```

This should produce a binary at:

```
target/aarch64-unknown-linux-musl/release/runtime-base-rust
```

### 4. Verify the Binary

Run the following to verify that the binary is an ELF 64-bit executable for ARM64 linked against musl:

```bash
file target/aarch64-unknown-linux-musl/release/runtime-base-rust
```

Expected output should include something like:

```
ELF 64-bit LSB pie executable, ARM aarch64, dynamically linked, interpreter /lib/ld-musl-aarch64.so.1, for GNU/Linux, not stripped
```

If the interpreter is `/lib/ld-musl-aarch64.so.1`, you have a musl-linked binary.

---

## Docker Deployment (Optional)

If you wish to deploy the runtime in a lightweight container (for example, on a Raspberry Pi or similar system), you can use an Alpine Linux–based image.

Create a `Dockerfile` in your project root with the following content:

```dockerfile
# Use an ARM64 Alpine Linux image (musl-based)
FROM arm64v8/alpine:latest

# Install necessary packages
RUN apk update && apk add --no-cache ca-certificates

# Create the module directory (permanent, so Survon finds it)
RUN mkdir -p /tmp/wasteland

# Copy the cross-compiled runtime into the container
COPY target/aarch64-unknown-linux-musl/release/runtime-base-rust /usr/local/bin/survon
RUN chmod +x /usr/local/bin/survon

# Set the default command to run the Survon runtime
CMD ["/usr/local/bin/survon"]
```

### Build and Run the Docker Container

From the project root (where the Dockerfile is located), build the image:

```bash
docker build -t survon-runtime .
```

Then run the container:

```bash
docker run -it survon-runtime
```

The runtime should start and, if no modules are present in `/tmp/wasteland`, display:

```
Module directory '/tmp/wasteland' does not exist.
Please provide modules to continue. Place module ZIP files in /tmp/wasteland and restart the application.
```

Since the Dockerfile now creates `/tmp/wasteland`, the runtime should find the directory even if it’s empty.

---

## Module Installation and Setup

Survon expects module packages as ZIP files placed in the `/tmp/wasteland` directory. Each module ZIP should have the following structure:

```
module-example.zip
├── manifest.json
└── libmodule_example.so
```

- **manifest.json:** Contains at least:
  ```json
  {
    "name": "Module Example",
    "lib_file": "libmodule_example.so"
  }
  ```
- **libmodule_example.so:** The dynamic library for the module.

### How to Install Modules

1. **Prepare your module ZIP** as described.
2. **Place the ZIP file in `/tmp/wasteland`:**
    - If using Docker with a mounted directory:
      ```bash
      docker run -it -v ~/modules:/tmp/wasteland survon-runtime
      ```
    - Or copy the ZIP file into the container if you're testing manually.
3. **Restart the Survon runtime.** It will scan `/tmp/wasteland`, extract the ZIP, and load the module.

---

## Troubleshooting

- **Cross-Compilation Issues:**  
  Verify that `.cargo/config.toml` is set correctly and that `aarch64-linux-musl-gcc` is installed.

- **Docker "No such file or directory" Error:**  
  If you see this error, ensure:
    - The binary is correctly copied to `/usr/local/bin/survon`.
    - The dynamic linker (from musl) is available in the base image. Using Alpine (which is musl-based) or creating the directory `/tmp/wasteland` in the Dockerfile can help.

- **Module Directory:**  
  The runtime automatically checks for `/tmp/wasteland`. We’ve set up the Dockerfile to create this directory. In non-container deployments, consider modifying the runtime to create the directory if it doesn’t exist.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
