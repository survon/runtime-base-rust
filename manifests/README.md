# Core Modules Vs Wasteland Modules

**Simply put**, core modules help you interact with the Survon OS, and wasteland modules help you interact with your homestead.

**More deeply put**, Survon comes with some built-in functionality. Core modules represent a way for you to interface with that functionality so the operating system can offer you some value from day one. Because we built them, they offer a lot more functionality than wasteland modules. They're much more like mini-apps: to-do list, calendar, knowledge base chat interface, etc.

Wasteland modules are a way for Survon to represent data from, and interface with, external hardware systems. The Survon system captures various inbound data from serial/radio/bluetooth/etc. It parses the packets and directs Survon-compatible event messages to the central message bus for parsing. The inbound data is redirected appropriately to its corresponding wasteland module so it can be displayed on the TUI. Each registered piece of hardware must have a corresponding wasteland module so we can both display its data as well as communicate back to it over the same payload contract. These wasteland modules assume that the IoT device in play has been configured to adhere to the Survon Serial Protocol (SSP).

## Example Wasteland Module Config

```yaml
# Auto-generated module for arduino_ble_001
# Device Type: field_unit
# Firmware: 1.0.0
# Location: modules/wasteland/arduino_ble_001/config.yml

# Sample SSP Telemetry Payload (device → Survon):
# Compact format - minimal bandwidth for BLE/Serial
# {
#   "p": "ssp/1.0",        # protocol version
#   "t": "tel",            # type: telemetry
#   "i": "arduino_ble_001", # device id (must match bus_topic)
#   "s": 1732377600,       # timestamp (seconds since epoch)
#   "d": {                 # data payload
#     "a": 3600,           # uptime_sec
#     "b": 1024,           # free_ram
#     "c": 512             # analog_a0
#   }
# }

# Sample SSP Command Payload (Survon → device):
# {
#   "p": "ssp/1.0",
#   "t": "cmd",            # type: command
#   "i": "arduino_ble_001",
#   "s": 1732377605,
#   "d": {
#     "action": "ping"     # command action
#   }
# }

# Sample SSP Capabilities Response (device → Survon on registration):
# Sent once when device first connects or when requested
# {
#   "p": "ssp/1.0",
#   "t": "res",            # type: response
#   "i": "arduino_ble_001",
#   "s": 1732377600,
#   "d": {
#     "dt": "field_unit",  # device_type
#     "fw": "1.0.0",       # firmware_version
#     "s": ["a", "b", "c"], # sensors (compact keys)
#     "a": ["blink"]       # actuators/actions
#   }
# }

# Module Configuration
name: "arduino_ble_001 (field_unit)"
module_type: "monitoring"
bus_topic: "arduino_ble_001"  # MUST match SSP "i" field
template: "status_badge_card"

# Module bindings - Auto-updated by Overseer from SSP telemetry
bindings:
  # Telemetry data (from SSP "d" object)
  # Keys match compact SSP format
  a: 0  # uptime_sec
  b: 0  # free_ram  
  c: 0  # analog_a0
  
  # Device metadata (from capabilities response)
  device_id: "arduino_ble_001"
  firmware_version: "1.0.0"
  is_blinkable: true
  
  # Status badge template bindings
  status: "online"
  message: "Field unit operational"
  count: 0  # Will display message count here
```

## Available Commands

These commands are defined by the device firmware and sent via SSP command messages:

- **ping**: Request immediate telemetry update
- **blink**: Trigger LED blink (if device supports it)

## Architecture Flow

1. **Device Discovery**: Hardware advertises via BLE/Serial → Discovery service detects it
2. **Trust Decision**: Overseer presents device to user → User trusts device via UI
3. **Registration**: Device sends capabilities response → Overseer auto-generates module config
4. **Telemetry Flow**: Device sends SSP telemetry → Message bus routes to module → Handler updates bindings → Template renders UI
5. **Command Flow**: User interacts with UI → Handler sends SSP command → Message bus routes to device

## Module Locations

- **Core Modules**: `modules/core/*/config.yml` (e.g., overseer, side_quest, survon_llm)
- **Wasteland Modules**: `modules/wasteland/*/config.yml` (e.g., a01, valve_control, monitoring_temp)
- **Module Handlers**: `src/modules/*/handler.rs` (implements ModuleHandler trait)
- **UI Templates**: `src/ui/template/module_templates/*/` (renders module interface)

## SSP Protocol Notes

**Compact vs Verbose Format**: The example above uses compact format (single-letter keys: "a", "b", "c") to minimize bandwidth for constrained devices. Verbose format uses descriptive keys ("uptime_sec", "free_ram", etc.) for clarity when bandwidth isn't a concern.

**Key Fields**:
- `p`: Protocol version (always "ssp/1.0")
- `t`: Message type ("tel"=telemetry, "cmd"=command, "res"=response)
- `i`: Device identifier (must match module's bus_topic)
- `s`: Timestamp (Unix seconds)
- `d`: Data payload (structure varies by message type)

**Handler Responsibilities**:
- Subscribe to bus_topic for incoming telemetry
- Parse SSP messages and update module bindings
- Send SSP commands when user interacts with UI
- Track connection status and timeouts
- Handle device registration on first connect

See `src/modules/monitoring/handler.rs` and `src/modules/valve_control/handler.rs` for implementation examples.
