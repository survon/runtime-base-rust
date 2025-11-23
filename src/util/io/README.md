# Survon Serial Protocol (SSP) v1.0

## Overview

SSP is a JSON-based protocol for bidirectional communication between the Survon hub (Raspberry Pi) and field devices (Arduino, ESP32, etc.). It's inspired by MQTT and JSON-RPC standards.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Survon Hub (Raspberry Pi)                 │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │   Modules    │◄───────►│ Message Bus  │                 │
│  │  (Pressure,  │         │ (BusMessage) │                 │
│  │   Gate, etc) │         └──────┬───────┘                 │
│  └──────────────┘                │                          │
│                                   │                          │
│  ┌────────────────────────────────┼──────────────────────┐  │
│  │        Transport Manager                              │  │
│  │                                                        │  │
│  │  • Converts: SspMessage ↔ BusMessage                │  │
│  │  • Routes by device_id + transport type              │  │
│  │  • Maintains routing table                           │  │
│  └───────────────────────┬────────────────────────────────┘  │
└──────────────────────────┼───────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
         ┌────▼───┐   ┌───▼────┐  ┌───▼────┐
         │  USB   │   │ LoRa   │  │  BLE   │
         │ Serial │   │ Radio  │  │        │
         └────┬───┘   └───┬────┘  └───┬────┘
              │           │           │
         (Arduino)   (Arduino)   (ESP32)
```

## Message Flow

### Inbound (Device → Survon)

1. Arduino reads sensor and creates SSP telemetry message
2. Sends JSON over serial (newline-delimited)
3. Transport Manager receives and parses SSP message
4. Stores device routing info (device_id → transport + address)
5. Converts to BusMessage and publishes to internal bus
6. Modules subscribe to topics and receive updates
7. UI updates with new sensor data

### Outbound (Survon → Device)

1. Module publishes command to internal bus (e.g., "open_gate")
2. Transport Manager subscribes to outbound topics
3. Extracts target device_id from payload
4. Looks up routing info for that device
5. Converts BusMessage to SSP command message
6. Sends via appropriate transport (USB/BLE/LoRa)
7. Device receives, processes, and sends SSP response

## Message Types

### 1. Telemetry (Sensor Data)

```json
{
  "protocol": "ssp/1.0",
  "type": "telemetry",
  "topic": "pressure_sensor",
  "timestamp": 1732377600,
  "source": {
    "id": "pressure_001",
    "transport": "usb",
    "address": "/dev/ttyUSB0"
  },
  "payload": {
    "pressure_psi": 85.5,
    "temperature_c": 22.3,
    "battery_pct": 87
  }
}
```

**Updates module binding**: `module.config.bindings.pressure_psi = 85.5`

### 2. Command (Control Actions)

```json
{
  "protocol": "ssp/1.0",
  "type": "command",
  "topic": "com_input",
  "timestamp": 1732377605,
  "source": {
    "id": "survon_hub",
    "transport": "internal",
    "address": "internal"
  },
  "payload": {
    "action": "open_gate",
    "duration_sec": 30
  },
  "reply_to": "gate_001"
}
```

### 3. Response (Acknowledgment)

```json
{
  "protocol": "ssp/1.0",
  "type": "response",
  "topic": "com_input",
  "timestamp": 1732377606,
  "source": {
    "id": "gate_001",
    "transport": "ble",
    "address": "00:1B:44:11:3A:B7"
  },
  "payload": {
    "status": "success",
    "action": "open_gate",
    "message": "Gate opened successfully"
  },
  "in_reply_to": "survon_hub"
}
```

### 4. Event (Status Changes)

```json
{
  "protocol": "ssp/1.0",
  "type": "event",
  "topic": "network",
  "timestamp": 1732377610,
  "source": {
    "id": "mesh_node_003",
    "transport": "lora",
    "address": "868.1"
  },
  "payload": {
    "event": "connection_established",
    "message": "Node connected to mesh network"
  }
}
```

**Appends to activity log**: `module.config.bindings.activity_log[]`

## Routing

The Transport Manager maintains a routing table:

```rust
HashMap<device_id, SourceInfo> {
  "pressure_001" => SourceInfo {
    id: "pressure_001",
    transport: Usb,
    address: "/dev/ttyUSB0"
  },
  "gate_001" => SourceInfo {
    id: "gate_001",
    transport: Ble,
    address: "00:1B:44:11:3A:B7"
  }
}
```

When sending commands:
1. Extract `device_id` or `target` from payload
2. Look up routing info
3. Send via correct transport + address

## Module Integration

### Pressure Gauge Module

```yaml
# config.yml
name: "Pressure Monitor"
module_type: "monitoring"
bus_topic: "pressure_sensor"
template: "gauge_card"
bindings:
  pressure_psi: 0.0
```

SSP telemetry automatically updates `pressure_psi` binding.

### Gate Control Module

```yaml
# config.yml
name: "Gate Control"
module_type: "com"
bus_topic: "com_input"
template: "toggle_switch"
bindings:
  state: false
  toggle_on_label: "Open"
  toggle_off_label: "Closed"
```

When user toggles, publishes to bus → Transport Manager routes to gate device.

## Implementation Checklist

- [x] SSP message format (src/util/ssp.rs)
- [x] Transport Manager (src/util/transport.rs)
- [x] Bus integration
- [x] Routing table
- [ ] USB serial I/O (requires `tokio-serial` crate)
- [ ] BLE transport (requires `bluez` bindings)
- [ ] LoRa/Radio transport (device-specific)

## Dependencies Needed

Add to `Cargo.toml`:

```toml
[dependencies]
tokio-serial = "5.4"  # For USB serial I/O
```

## Testing

1. **Without hardware**: Current stub mode logs messages
2. **With USB device**: Connect Arduino running example code
3. **Live testing**: `DEBUG=true cargo run` to see SSP messages in logs

## Arduino Libraries Required

```cpp
#include <ArduinoJson.h>  // Version 6+
```

Install via Arduino Library Manager.

## Performance

- **Bandwidth**: 115200 baud = ~11.5 KB/s = ~100 JSON messages/sec
- **Latency**: <10ms for USB, varies by transport
- **Overhead**: ~200 bytes per message (JSON + metadata)

## Security Considerations

- No authentication in v1.0 (devices trusted on local network)
- Future: Add HMAC signatures or encrypted payloads
- Physical security: USB/serial connections are local-only

## Future Enhancements

- Message compression (MessagePack instead of JSON)
- QoS levels (guaranteed delivery)
- Retain flag (last-known-good values)
- Device discovery/announcement protocol
- Firmware update over SSP
