# Survon Base Runtime

This is a Ratatui-based terminal UI application for Survon, an offline modular survival system focused on resilience for off-grid scenarios.

## System Architecture

Survon is designed as a modular IoT hub that bridges physical hardware with a unified terminal interface. The system intercepts sensor data and device commands from various sources—USB peripherals, Bluetooth devices, radio modules, and custom IoT hardware—and presents them through a real-time TUI.

**Example use cases:**
- Remote gate control: An Arduino-based controller sends open/close commands via radio to the Survon hub
- Environmental monitoring: Pressure sensors, temperature gauges, and water flow meters transmit data wirelessly from distributed locations around the homestead
- Equipment status: Monitor well pumps, generators, or greenhouse automation systems through standardized Survon-compatible payloads

The runtime uses a message bus architecture to route events between hardware modules and UI components. Compatible hardware can be built using Survon protocol adapters (or third-party implementations) on microcontrollers like Arduino, ESP32, or similar platforms. These devices broadcast sensor readings and accept commands via radio (LoRa, 433MHz), Bluetooth, or direct serial connection to the Raspberry Pi hub.

All sensor data, control commands, and system events flow through the central message bus, enabling real-time monitoring and control of homestead infrastructure from a single interface.

## Architecture Summary
```
┌───────────────────────────────────────────────────────────┐
│                        Survon Hub                         │
│                                                           │
│  ┌──────────────┐         ┌──────────────┐                │
│  │   Modules    │◄───────►│ Message Bus  │                │
│  │  (Pressure,  │         │ (BusMessage) │                │
│  │   Gate, etc) │         └──────┬───────┘                │
│  └──────────────┘                │                        │
│                                  │                        │
│  ┌───────────────────────────────┼─────────────────────┐  │
│  │        Serial/Transport Manager                     │  │
│  │                                                     │  │
│  │  Inbound:  SspMessage → BusMessage                  │  │
│  │  Outbound: BusMessage → SspMessage                  │  │
│  │                                                     │  │
│  │  Routing Table: device_id → (transport, address)    │  │
│  └───────────────────────┬─────────────────────────────┘  │
└──────────────────────────┼────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
         ┌────▼───┐    ┌───▼────┐   ┌───▼────┐
         │  BLE   │    │ Radio  │   │  USB   │
         └────┬───┘    └───┬────┘   └───┬────┘
              │            │            │
         (Arduino)    (Arduino)    (Arduino)
```

## Installation
Clone and build manually, or use the Survon OS installer for RPi 3B (armhf):
```bash
curl -sSL https://raw.githubusercontent.com/survon/survon-os/master/scripts/install.sh | bash
```
- Installer compiles llama-cli from llama.cpp, downloads models (interactive: phi3-mini or custom URL), builds release binary.
- Post-install: Reboot for bash menu; launch app via option 4 (takes over terminal like a full-screen React component).

## Mac Development: Configure [LE Friend Dongle](https://www.adafruit.com/product/2267) for Auto-Connect (One-Time)

When developing on macOS, the LE Friend dongle does **not** auto-connect by default. The [Survon OS](https://github.com/survon/survon-os) installer 
will send the serial commands to the dongle automatically. But, since we're building the Survon Runtime on a Mac and didn't "install" it, 
we need to reproduce this critical step.

Thankfully, you only need to do this once per dongle. Ensure the LE Friend dongle is plugged in and blinking, 
and run the following in a terminal to permanently enable auto-connect:

```bash
PORT=$(ls /dev/cu.usbserial-* 2>/dev/null | head -n1)
[ -z "$PORT" ] && { echo "No LE Friend found"; exit 1; }
echo "Configuring $PORT ..."
stty -f "$PORT" 57600 raw -echo
(echo -e "AT+HWREGWRITE=0x01,0x01\r"; sleep 1; echo -e "ATZ\r") > "$PORT"
sleep 2
echo "Done — auto-connect enabled forever"
```

## Usage
- Development: `cargo run` or `./target/release/runtime-base-rust`
- Production (post-install): `/usr/local/bin/runtime-base-rust` or via Survon OS menu option 4
- Config: Edit via menu option 2 (sets `~/.bashrc`; source for immediate use)
- The installer downloads a pre-built armv7 binary from GitHub releases (no compilation needed on Pi)

## Debug Logging
The application writes logs to `./logs/` directory with separate files for each severity level:
- `error.log` - Error messages
- `warn.log` - Warning messages
- `info.log` - Informational messages
- `debug.log` - Verbose debug output (only when DEBUG flag is enabled)

To enable debug logging:
```bash
# Development
DEBUG=true cargo run

# Production (direct execution)
DEBUG=true /usr/local/bin/runtime-base-rust
```

Debug logs are useful for troubleshooting issues, especially on embedded devices like Raspberry Pi. Log files are cleared on each startup to prevent disk space issues.

## Production Deployment
When running on Survon OS (Raspberry Pi), enable debug logging by setting the environment variable before launch.

**Option 1: Via survon.sh menu**
Edit environment variables (menu option 2) and add:
```bash
# In survon.sh menu:
Set ENV_VAR: DEBUG
Value: true
```

**Option 2: Modify survon.sh directly**
Edit `/home/survon/survon.sh` option 4 to include the DEBUG flag:
```bash
4) # Launch Rust TUI
   cd /home/survon
   DEBUG=true /usr/local/bin/runtime-base-rust
   ;;
```

**Option 3: Custom startup script**
Create a wrapper script that sets DEBUG conditionally:
```bash
#!/bin/bash
# /usr/local/bin/start-survon.sh
DEBUG=true /usr/local/bin/runtime-base-rust
```

The DEBUG flag works identically in release builds - environment variables are checked at runtime, not compile time.

## License
Copyright (c) Sean Cannon

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>).
