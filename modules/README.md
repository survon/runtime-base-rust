# Core Modules Vs Wasteland Modules

**Simply put**, core modules help you interact with the Survon OS, and wasteland modules help you interact with your homestead.

**More deeply put**, Survon comes with some built-in functionality. Core modules represent a way for you to interface with that functionality so 
the operating system can offer you some value from day one. Because we built them, they offer a lot more functionality than 
wasteland modules. They're much more like mini-apps: to-do list, calendar, knowledge base chat interface, etc. 

Wasteland modules are a way for Survon to represent data from, and interface with, external hardware systems. The Survon system 
captures various inbound data from serial/radio/bluetooth/etc. It parses the packets and directs Survon-compatible event messages
to the central message bus for parsing. The inbound data is redirected appropriately to its corresponding wasteland module so it can 
be displayed on the TUI. Each registered piece of hardware must have a corresponding wasteland module so we can both display its data 
as well as communicate back to it over the same payload contract. These wasteland modules assume that the IoT device in play has 
been configured to adhere to the Survon serial bus contract. 

```
# Auto-generated module for arduino_ble_001
# Device Type: field_unit
# Firmware: 1.0.0

# Sample SSP Telemetry Payload:
# {
#   "protocol": "ssp/1.0",
#   "type": "telemetry",
#   "topic": "arduino_ble_001",
#   "timestamp": 1732377600,
#   "source": {
#     "id": "arduino_ble_001",
#     "transport": "ble",
#     "address": "12:34:56:78:9A:BC"
#   },
#   "payload": {
#     "uptime_sec": 3600,
#     "free_ram": 1024,
#     "analog_a0": 512,
#     "ping_count": 42
#   }
# }

# Sample SSP Command Payload (to send commands to this device):
# {
#   "protocol": "ssp/1.0",
#   "type": "command",
#   "topic": "arduino_ble_001",
#   "timestamp": 1732377605,
#   "source": {
#     "id": "survon_hub",
#     "transport": "internal",
#     "address": "internal"
#   },
#   "payload": {
#     "action": "ping"
#   },
#   "reply_to": "arduino_ble_001"
# }

# Available Commands:
# - ping: Request immediate telemetry update
# - reset: Reset ping counter to zero
# - status: Request device status message

name: "arduino_ble_001 (field_unit)"
module_type: "monitoring"
bus_topic: "arduino_ble_001"
template: "status_badge_card"
bindings:
  # Auto-updated by SSP telemetry messages
  uptime_sec: 0
  free_ram: 0
  analog_a0: 0
  ping_count: 0
  
  # Device metadata
  device_id: "arduino_ble_001"
  firmware_version: "1.0.0"
  is_blinkable: true
  
  # Status badge specific bindings
  status: "online"
  message: "Field unit operational"
  count: 0  # Will display ping_count here
```
