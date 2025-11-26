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




# Survon Serial Protocol (SSP) - Compact Format

## Overview

SSP Compact is a memory-efficient JSON protocol for IoT devices with limited RAM (like Arduino Uno). Single-letter keys minimize message size while maintaining structure.

## Message Format

```json
{
  "p": "ssp/1.0",
  "t": "tel",
  "i": "device_id",
  "s": 1234567890,
  "d": {
    "a": 82,
    "b": 26,
    "c": 42
  }
}
```

## Field Definitions

| Key | Full Name   | Type   | Description                           | Example         |
|-----|-------------|--------|---------------------------------------|-----------------|
| `p` | protocol    | string | Protocol version                      | `"ssp/1.0"`     |
| `t` | type        | string | Message type (see below)              | `"tel"`         |
| `i` | id          | string | Device identifier (also used as topic)| `"a01"`         |
| `s` | timestamp   | number | Unix timestamp or millis/1000         | `1234567890`    |
| `d` | data        | object | Sensor/payload data (see below)       | `{"a":82}`      |

## Message Types (`t`)

| Short | Full      | Description                    |
|-------|-----------|--------------------------------|
| `tel` | telemetry | Sensor readings                |
| `cmd` | command   | Control command                |
| `res` | response  | Command acknowledgment         |
| `evt` | event     | Status change notification     |

## Data Payload (`d`)

**Keys are single letters** that map to meaningful names in `config.yml`:

```yaml
# modules/a01/config.yml
name: "Temperature Sensor"
bus_topic: "a01"  # Must match "i" field
bindings:
  a: 0  # temperature_c
  b: 0  # humidity_pct
  c: 0  # message_count
```

### Common Data Key Conventions

| Key | Suggested Use        | Unit/Type    |
|-----|---------------------|--------------|
| `a` | Primary sensor      | varies       |
| `b` | Secondary sensor    | varies       |
| `c` | Counter/sequence    | integer      |
| `d` | Tertiary sensor     | varies       |
| `e` | Error code          | integer      |
| `f` | Battery/power       | percent      |

**Important:** Keys are device-specific. Define mappings in your module's `config.yml`.

## Arduino Example

```cpp
void sendTelemetry() {
  StaticJsonDocument<128> doc;
  
  doc["p"] = "ssp/1.0";
  doc["t"] = "tel";
  doc["i"] = "a01";
  doc["s"] = millis() / 1000;
  
  JsonObject data = doc.createNestedObject("d");
  data["a"] = (int)readTemperature();
  data["b"] = (int)readHumidity();
  data["c"] = messageCount++;

  String json;
  serializeJson(doc, json);
  
  // Send via BLE in 20-byte chunks
  ble.setMode(BLUEFRUIT_MODE_DATA);
  for (size_t i = 0; i < json.length(); i += 20) {
    String chunk = json.substring(i, min(i + 20, json.length()));
    ble.print(chunk);
    delay(10);
  }
  ble.setMode(BLUEFRUIT_MODE_COMMAND);
}
```

Expected output: `{"p":"ssp/1.0","t":"tel","i":"a01","s":1234,"d":{"a":72,"b":45,"c":1}}`

Size: **~68 bytes** vs **~180 bytes** for verbose format

## Module Configuration

### Step 1: Create module directory
```bash
mkdir -p modules/wasteland/a01
```

### Step 2: Create `config.yml`
```yaml
name: "Environmental Monitor"
module_type: "monitoring"
bus_topic: "a01"      # Must match device "i" field
template: "gauge_card"

bindings:
  # Map data keys to display names
  a: 0  # temperature_c - shown in gauge
  b: 0  # humidity_pct - secondary display
  c: 0  # message_count - debug counter
  
  # Metadata
  device_id: "a01"
  unit_of_measure_label: "°C"
  display_name: "Temperature"
```

### Step 3: Update template (if needed)

Edit your template to use the data keys:

```rust
// src/ui/module_templates/monitoring/gauge_card.rs

const GAUGE_VALUE_KEY: &str = "a";  // Maps to temperature_c

let value = module
    .config
    .bindings
    .get(GAUGE_VALUE_KEY)
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0);
```

## Message Flow

```
┌─────────────┐
│  Arduino    │ {"p":"ssp/1.0","t":"tel","i":"a01","s":100,"d":{"a":72}}
└──────┬──────┘
       │ BLE chunks (20 bytes each)
       ▼
┌─────────────┐
│ Discovery   │ Reassembles chunks → complete JSON
│ Manager     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ parse_      │ Extracts: topic="a01", payload={"a":72}
│ flexible()  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Message Bus │ Publishes to topic "a01"
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Module a01  │ Updates binding: a=72
│ (gauge_card)│ Renders: "72.0 °C"
└─────────────┘
```

## Size Comparison

| Format  | Example Message                                                              | Size   |
|---------|------------------------------------------------------------------------------|--------|
| Verbose | `{"protocol":"ssp/1.0","type":"telemetry","topic":"sensor_001",...}`        | 180 bytes |
| Compact | `{"p":"ssp/1.0","t":"tel","i":"a01","s":100,"d":{"a":72,"b":45,"c":1}}`    | 68 bytes  |

**Savings:** 62% smaller → fits in Arduino Uno RAM + faster BLE transmission

## Debugging

Enable debug logging in Rust to see parsing:

```rust
// Logs will show:
[DEBUG] Parsing compact SSP: {"p":"ssp/1.0","t":"tel",...}
[DEBUG] ✓ Parsed compact SSP - id:a01, type:telemetry, data keys:["a","b","c"]
[INFO] 📨 Parsed SSP message: topic=a01, type=Telemetry
[INFO] ✓ Published to message bus: topic=a01
```

On Arduino Serial Monitor:
```
TELEM #1
TX_LEN:68
TX:{"p":"ssp/1.0","t":"tel","i":"a01","s":24,"d":{"a":82,"b":26,"c":1}}
TX_OK
RAM:1243
```

## Best Practices

1. **Keep device IDs short:** Use 2-4 characters (e.g., `"a01"`, `"tmp1"`)
2. **Document your mappings:** Always add comments in `config.yml` explaining what `a`, `b`, `c` mean
3. **Reserve keys:**
    - `a`-`d`: Sensor values
    - `e`: Error codes
    - `f`: Battery/health metrics
4. **Use integers when possible:** `{"a":82}` instead of `{"a":82.5}` saves bytes
5. **Timestamp efficiency:** Use `millis()/1000` instead of full Unix timestamp

## Future Enhancements

- [ ] Binary format (even smaller)
- [ ] Compression for multi-sensor devices
- [ ] Auto-discovery broadcasts capabilities in compact format
- [ ] Hub sends configuration updates to devices
