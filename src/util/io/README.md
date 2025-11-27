# Survon Serial Protocol (SSP) v1.0 - Compact Format

## Overview

SSP is a lightweight, memory-efficient JSON protocol for bidirectional communication between the Survon hub and IoT field devices (Arduino, ESP32, etc.). The compact format uses single-letter keys to minimize RAM usage on resource-constrained devices.

**Design Philosophy:**
- **Compact by default**: 68 bytes vs 180 bytes (62% smaller)
- **Agnostic sensors**: Devices send generic data keys (`a`, `b`, `c`); the hub's module config maps them to meaningful names
- **Self-documenting**: Module `config.yml` files serve as the schema definition

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Survon Hub (Raspberry Pi)                 │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │   Modules    │◄───────►│ Message Bus  │                 │
│  │   (a01,      │         │              │                 │
│  │    temp1)    │         └──────┬───────┘                 │
│  └──────────────┘                │                          │
│         ▲                        │                          │
│         │                        ▼                          │
│  ┌──────┴──────────────────────────────────────────────┐   │
│  │        Discovery/Transport Manager                   │   │
│  │  • Parses compact SSP messages                       │   │
│  │  • Routes by device_id (topic)                       │   │
│  │  • Publishes to message bus                          │   │
│  └──────────────────────┬───────────────────────────────┘   │
└─────────────────────────┼───────────────────────────────────┘
                          │
              ┌───────────┼───────────┐
              │           │           │
         ┌────▼───┐  ┌───▼────┐ ┌───▼────┐
         │  USB   │  │  BLE   │ │ LoRa   │
         │ Serial │  │        │ │        │
         └────┬───┘  └───┬────┘ └───┬────┘
              │          │          │
         (Arduino)  (ESP32)    (Radio)
```

---

## Message Format

### Compact SSP Structure

```json
{
  "p": "ssp/1.0",
  "t": "tel",
  "i": "a01",
  "s": 1234567890,
  "d": {
    "a": 82,
    "b": 26,
    "c": 1
  }
}
```

### Field Reference

| Key | Full Name   | Type   | Description                                    | Example         |
|-----|-------------|--------|------------------------------------------------|-----------------|
| `p` | protocol    | string | Protocol version (always `"ssp/1.0"`)          | `"ssp/1.0"`     |
| `t` | type        | string | Message type (see types below)                 | `"tel"`         |
| `i` | id          | string | Device identifier (used as bus topic)          | `"a01"`         |
| `s` | timestamp   | number | Unix timestamp or `millis()/1000`              | `1234567890`    |
| `d` | data        | object | Sensor/payload data with single-letter keys    | `{"a":82}`      |

### Message Types

| Short | Full      | Description                    | Use Case                        |
|-------|-----------|--------------------------------|---------------------------------|
| `tel` | telemetry | Sensor readings                | Temperature, humidity, pressure |
| `cmd` | command   | Control command                | Open gate, turn on LED          |
| `res` | response  | Command acknowledgment         | Success/failure confirmation    |
| `evt` | event     | Status change notification     | Connection established, error   |

---

## Message Flow

### Inbound (Device → Hub)

```
1. Arduino creates compact JSON:
   {"p":"ssp/1.0","t":"tel","i":"a01","s":100,"d":{"a":72,"b":45,"c":1}}

2. Sends via BLE/USB (automatically chunked into 20-byte packets)

3. Discovery Manager reassembles chunks → complete JSON

4. parse_flexible() extracts:
   - topic: "a01"
   - payload: {"a":72, "b":45, "c":1}

5. Message Bus publishes to topic "a01"

6. Module with bus_topic: "a01" receives update

7. Module bindings updated:
   - a → temperature_c = 72
   - b → humidity_pct = 45
   - c → message_count = 1

8. UI renders: "72.0 °C"
```

### Outbound (Hub → Device)

```
1. Module publishes command to bus (e.g., "com_input")

2. Transport Manager:
   - Extracts target device_id from payload
   - Looks up routing info (transport + address)
   - Converts to SSP format

3. Sends via appropriate transport (USB/BLE/LoRa)

4. Device receives, processes, and sends response
```

---

## Data Payload Conventions

The `"d"` (data) object uses **single-letter keys** that are device-specific. The hub's module `config.yml` maps these to human-readable names.

### Recommended Key Conventions

| Key | Suggested Use              | Notes                                    |
|-----|----------------------------|------------------------------------------|
| `a` | Primary sensor value       | Temperature, pressure, main reading      |
| `b` | Secondary sensor value     | Humidity, secondary reading              |
| `c` | Message counter / sequence | Health indicator, debug aid              |
| `d` | Tertiary sensor            | Battery, third sensor                    |
| `e` | Error code                 | 0 = OK, >0 = specific error              |
| `f` | Battery / power level      | Percentage or voltage                    |

**Important:** Keys are **not standardized** across devices. Each device defines its own schema, documented in its module config.

---

## Arduino Implementation

### Complete Example

```cpp
#include <ArduinoJson.h>
#include <Adafruit_BLE.h>
#include <Adafruit_BluefruitLE_SPI.h>

Adafruit_BluefruitLE_SPI ble(8, 7, 4);
int messageCount = 0;

void setup() {
  Serial.begin(115200);
  ble.begin();
  ble.setMode(BLUEFRUIT_MODE_COMMAND);
  ble.sendCommandCheckOK("AT+GAPDEVNAME=Survon Field Unit");
  ble.sendCommandCheckOK("AT+GAPSTARTADV");
}

void loop() {
  sendTelemetry();
  delay(3000);
}

void sendTelemetry() {
  messageCount++;
  
  // Read sensors
  float temp = readTemperature();
  float humid = readHumidity();
  
  // Create compact SSP message
  StaticJsonDocument<128> doc;
  doc["p"] = "ssp/1.0";
  doc["t"] = "tel";
  doc["i"] = "a01";
  doc["s"] = millis() / 1000;
  
  JsonObject data = doc.createNestedObject("d");
  data["a"] = (int)temp;
  data["b"] = (int)humid;
  data["c"] = messageCount;

  String json;
  serializeJson(doc, json);
  
  Serial.print(F("TX:"));
  Serial.println(json);
  
  // Send via BLE (let module handle chunking)
  ble.setMode(BLUEFRUIT_MODE_DATA);
  ble.print(json);
  delay(100);  // Allow transmission
  ble.setMode(BLUEFRUIT_MODE_COMMAND);
  
  Serial.println(F("TX_OK"));
}
```

### Expected Serial Output

```
TELEM #1
TX:{"p":"ssp/1.0","t":"tel","i":"a01","s":24,"d":{"a":72,"b":45,"c":1}}
TX_OK
RAM:1243
```

**Message size:** ~68 bytes

---

## Hub Configuration

### Step 1: Create Module Directory

```bash
mkdir -p modules/wasteland/a01
```

### Step 2: Create `config.yml`

```yaml
# modules/wasteland/a01/config.yml
name: "Environmental Monitor"
module_type: "monitoring"
bus_topic: "a01"      # Must match device "i" field
template: "gauge_card"

bindings:
  # Map single-letter keys to meaningful names
  a: 0  # temperature_c - primary sensor (displayed)
  b: 0  # humidity_pct - secondary sensor
  c: 0  # message_count - health counter
  
  # Display configuration
  device_id: "a01"
  unit_of_measure_label: "°C"
  display_name: "Temperature"
  
  # Gauge thresholds
  max_value: 100.0
  warn_threshold: 60.0
  danger_threshold: 85.0
  
  # Metadata
  firmware_version: "1.0.0"
  is_blinkable: true
```

### Step 3: Whitelist Topic (Temporary - Auto-registration coming)

In `app.rs`, add the device topic to outbound topics:

```rust
transport_manager.add_outbound_topic("a01".to_string()).await;
```

**Note:** This will be automated through device registration in the future.

---

## Template Integration

Templates access data using the single-letter keys:

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

The module's `config.yml` documents that `"a"` represents temperature, but the template just uses the key directly.

---

## Debugging

### Rust Hub Logs

```
[INFO] 🔷 BLE listener task started for 00:00:00:00:00:00
[INFO] ✓ Got notification stream for 00:00:00:00:00:00
[INFO] 📦 Chunk #1: 20 bytes: '{"p":"ssp/1.0","t":'
[INFO] 📦 Chunk #2: 20 bytes: '"tel","i":"a01","s"'
[INFO] 📦 Chunk #3: 28 bytes: ':24,"d":{"a":72,"b":45,"c":1}}'
[INFO] ✅ COMPLETE MESSAGE (68 bytes): {"p":"ssp/1.0","t":"tel","i":"a01","s":24,"d":{"a":72,"b":45,"c":1}}
[DEBUG] Parsing compact SSP: {"p":"ssp/1.0",...}
[DEBUG] ✓ Parsed compact SSP - id:a01, type:telemetry, data keys:["a","b","c"]
[INFO] 📨 Parsed SSP message: topic=a01, type=Telemetry
[INFO] 🚀 Publishing to bus - topic:a01, source:a01
[INFO] ✅ Published successfully!
```

### Arduino Serial Monitor

```
TELEM #1
TX_LEN:68
TX:{"p":"ssp/1.0","t":"tel","i":"a01","s":24,"d":{"a":72,"b":45,"c":1}}
TX_OK
RAM:1243
```

---

## Size Comparison

| Format     | Example                                                          | Size      |
|------------|------------------------------------------------------------------|-----------|
| **Verbose**| `{"protocol":"ssp/1.0","type":"telemetry","topic":"sensor_001","timestamp":1234567890,"source":{"id":"sensor_001","transport":"ble","address":""},"payload":{"temperature_c":72,"humidity_pct":45,"message_count":1}}` | **~180 bytes** |
| **Compact**| `{"p":"ssp/1.0","t":"tel","i":"a01","s":1234567890,"d":{"a":72,"b":45,"c":1}}` | **~68 bytes** |

**Savings:** 62% reduction → fits in Arduino Uno RAM, faster BLE transmission

---

## Best Practices

### For Device Developers

1. **Keep device IDs short:** 2-4 characters (e.g., `"a01"`, `"tmp1"`)
2. **Use integers when possible:** `{"a":82}` instead of `{"a":82.5}` saves bytes
3. **Timestamp efficiency:** Use `millis()/1000` for relative time
4. **Memory management:** Pre-allocate `StaticJsonDocument<128>` for compact format
5. **Test JSON validity:** Use Serial Monitor to verify output before BLE testing

### For Hub Configuration

1. **Document your mappings:** Always comment what `a`, `b`, `c` mean in `config.yml`
2. **Consistent conventions:** Within a project, try to use `a` for primary sensor, `b` for secondary, etc.
3. **Module naming:** Use descriptive names that match the device's purpose
4. **Topic uniqueness:** Each device needs a unique `bus_topic` matching its `"i"` field

---

## Troubleshooting

### Issue: No messages in UI

**Check:**
1. Module directory exists: `ls modules/wasteland/a01/config.yml`
2. `bus_topic` matches device's `"i"` field
3. Topic is whitelisted (temp fix until auto-registration)
4. BLE connection established (check logs for "✓ Got notification stream")

### Issue: Malformed JSON / Extra braces

**Cause:** Buffer overflow or improper serialization

**Fix:**
- Remove manual chunking loops
- Let BLE module handle transmission: `ble.print(json)` with delay
- Verify JSON with `serializeJson()` output in Serial Monitor

### Issue: Stream dies after 1 minute

**Cause:** BLE notification stream timeout

**Fix:**
- Ensure continuous data flow (send every 2-3 seconds)
- Check for Arduino crashes (monitor `getFreeRam()`)
- Verify BLE module firmware is up to date

---

## Performance Metrics

| Metric              | Value                          |
|---------------------|--------------------------------|
| **Bandwidth**       | ~11.5 KB/s @ 115200 baud       |
| **Latency**         | <50ms (BLE), <10ms (USB)       |
| **Message overhead**| 40 bytes (protocol + metadata) |
| **Data payload**    | 28 bytes (3 sensors)           |
| **Total size**      | 68 bytes                       |
| **Messages/second** | ~170 theoretical, ~10 practical|

---

## Future Enhancements

- [ ] **Auto-registration**: Devices broadcast capabilities on connect
- [ ] **Binary format**: Even smaller (MessagePack or CBOR)
- [ ] **QoS levels**: Guaranteed delivery for commands
- [ ] **Compression**: For multi-sensor devices (10+ readings)
- [ ] **OTA updates**: Firmware updates via SSP
- [ ] **Discovery protocol**: Automatic topic whitelisting
- [ ] **Schema validation**: Hub validates `"d"` keys against module config

---

## Security Considerations

**Current (v1.0):**
- No authentication (devices trusted on local network/BLE pairing)
- Physical security: USB/BLE connections are short-range

**Future:**
- HMAC signatures for message integrity
- Encrypted payloads for sensitive data
- Device certificates for authentication

---

## Dependencies

### Arduino
```cpp
#include <ArduinoJson.h>  // Version 7+
```
Install via Arduino Library Manager

### Rust (Hub)
```toml
[dependencies]
tokio-serial = "5.4"
btleplug = "0.11"
serde_json = "1.0"
```

---

## License

Survon Serial Protocol is part of the Survon project.

---

## Appendix: Complete Device Schema Example

```yaml
# Documentation for device "a01"
# This serves as the authoritative schema definition

device_id: "a01"
device_name: "Environmental Monitor v1"
firmware: "1.0.0"
hardware: "Arduino Uno + DHT22 + BLE"

# SSP Message Structure
message_format:
  p: "ssp/1.0"
  t: "tel"
  i: "a01"
  s: <timestamp>
  d:
    a: <int>  # temperature_c (range: -40 to 85)
    b: <int>  # humidity_pct (range: 0 to 100)
    c: <int>  # message_count (incrementing)

# Hub Module Config
module_config: ./modules/wasteland/a01/config.yml

# Update Frequency
interval: 3000ms
```

This schema lives in your project documentation and ensures anyone working with the device understands the data structure.
