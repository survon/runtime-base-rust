# SSP Valve Control - Complete Testing Guide

## Quick Reference: SSP Message Format

### Telemetry (Arduino → Hub)
```json
{
  "p": "ssp/1.0",
  "t": "tel",
  "i": "v01",
  "s": 1234567890,
  "d": {
    "a": 1,    // valve_open: 0=closed, 1=open
    "b": 95,   // position: 0-100%
    "c": 123   // message_count
  }
}
```

### Command (Hub → Arduino)
```json
{
  "p": "ssp/1.0",
  "t": "cmd",
  "i": "v01",
  "s": 1234567890,
  "d": {
    "action": "open"  // or "close"
  }
}
```

### Capabilities (Arduino → Hub on registration)
```json
{
  "p": "ssp/1.0",
  "t": "res",
  "i": "v01",
  "s": 1234567890,
  "d": {
    "dt": "valve_ctrl",
    "fw": "1.0.0",
    "s": ["a", "b", "c"],
    "a": ["valve"]
  }
}
```

---

## Setup Checklist

### Hardware
- [ ] Arduino Uno connected to computer via USB
- [ ] Adafruit Bluefruit LE SPI Shield installed
- [ ] Servo connected to Pin 9 (signal)
- [ ] Servo power (5V for SG90, external 6V for MG996R)
- [ ] Common ground between Arduino and servo power
- [ ] Optional: Status LED on Pin 13

### Software
- [ ] Arduino IDE or arduino-cli installed
- [ ] ArduinoJson library (v7+) installed
- [ ] Adafruit BluefruitLE library installed
- [ ] Servo library (comes with Arduino IDE)

---

## Step-by-Step Testing

### Phase 1: Upload and Verify Arduino (5 min)

1. **Upload the sketch**
   ```bash
   # Using Arduino IDE:
   File → Open → valve_control_ssp.ino
   Tools → Board → Arduino Uno
   Tools → Port → (your port)
   Upload
   ```

2. **Open Serial Monitor**
   ```
   Tools → Serial Monitor
   Set baud rate: 115200
   ```

3. **Expected startup output:**
   ```
   === Survon Valve Controller (SSP) ===
   Initializing Bluefruit...
   ✓ Bluefruit initialized
   MAC Address: XX:XX:XX:XX:XX:XX
   ✓ Advertising as 'Survon Valve'
   ✓ Valve initialized (CLOSED)
   
   TELEM #1
   TX_LEN: 68
   TX: {"p":"ssp/1.0","t":"tel","i":"v01","s":24,"d":{"a":0,"b":0,"c":1}}
   TX_OK
   RAM: 1243
   ```

4. **Test manual commands:**
   ```
   Type in Serial Monitor:
   
   open      ← Should move servo to 90°
   STATUS    ← Shows current state
   close     ← Should move servo to 0°
   CAP       ← Sends capabilities message
   ```

5. **Verify servo movement:**
    - Servo should move smoothly (not jittery)
    - Should take ~1.5 seconds for full rotation
    - Should stop at correct angles

**✓ Phase 1 Complete:** Arduino is working independently

---

### Phase 2: Configure Module (3 min)

1. **Create module directory:**
   ```bash
   mkdir -p modules/wasteland/valve_control
   ```

2. **Create config.yml:**
   ```yaml
   # modules/wasteland/valve_control/config.yml
   name: "Valve Control"
   module_type: "valve_control"
   bus_topic: "v01"
   template: "toggle_switch"
   
   bindings:
     a: 0  # valve_open
     b: 0  # position
     c: 0  # message_count
     
     device_id: "v01"
     device_type: "valve_ctrl"
     firmware_version: "1.0.0"
     
     state: false
     label: "Primary Flow Control"
     toggle_on_label: "Open"
     toggle_off_label: "Closed"
     description: "Controls pipeline flow"
     is_blinkable: false
   ```

3. **Verify file structure:**
   ```bash
   tree modules/wasteland/
   
   modules/wasteland/
   └── valve_control
       └── config.yml
   ```

**✓ Phase 2 Complete:** Module configured

---

### Phase 3: Integrate Handler (10 min)

1. **Create handler directory:**
   ```bash
   mkdir -p src/modules/valve_control
   ```

2. **Add handler.rs:**
    - Copy code from "Valve Control Handler" artifact
    - Save as `src/modules/valve_control/handler.rs`

3. **Create mod.rs:**
   ```bash
   echo "pub mod handler;" > src/modules/valve_control/mod.rs
   ```

4. **Update src/modules/mod.rs:**
   ```rust
   pub mod llm;
   pub mod module_handler;
   pub mod overseer;
   pub mod valve_control;  // <-- ADD THIS
   ```

5. **Register handler in ModuleManager:**

   In `src/modules/mod.rs`, find `initialize_module_handlers()` and add:

   ```rust
   "valve_control" => {
       if !self.handlers.contains_key("valve_control") {
           use crate::modules::valve_control;
           
           if let Some(valve_module) = self.modules.iter()
               .find(|m| m.config.module_type == "valve_control") 
           {
               let device_id = valve_module.config.bindings
                   .get("device_id")
                   .and_then(|v| v.as_str())
                   .unwrap_or("v01")
                   .to_string();
               
               let bus_topic = valve_module.config.bus_topic.clone();
               
               let handler = Box::new(
                   valve_control::handler::ValveControlHandler::new(
                       message_bus.clone(),
                       device_id,
                       bus_topic,
                   )
               );
               self.register_handler(handler);
           }
       }
   }
   ```

6. **Compile:**
   ```bash
   cargo build
   ```

**✓ Phase 3 Complete:** Handler integrated

---

### Phase 4: BLE Discovery (5 min)

1. **Start Survon:**
   ```bash
   cargo run
   ```

2. **Watch logs for BLE discovery:**
   ```
   INFO: Scanning for Survon field units...
   INFO: Found Survon device: Survon Valve (XX:XX:XX:XX:XX:XX), RSSI: -55
   INFO: 🆕 NEW device XX:XX:XX:XX:XX:XX discovered, awaiting trust decision
   ```

3. **Open Wasteland Manager in UI:**
    - Navigate to "Wasteland Manager" module
    - Select "Trust Pending Devices"
    - You should see: `Survon Valve (XX:XX:XX:XX:XX:XX) RSSI: -55 dBm`

4. **Trust the device:**
    - Select the valve device
    - Press Enter to trust
    - Wait for registration

5. **Watch logs for registration:**
   ```
   INFO: ✓ Connected to XX:XX:XX:XX:XX:XX
   INFO: ✓ Discovered services
   INFO: ✓ Found RX characteristic
   INFO: ✓ Subscribed to notifications
   INFO: 📦 Chunk #1: ...
   INFO: ✅ COMPLETE MESSAGE (120 bytes)
   INFO: 🔷 Detected COMPACT registration format
   INFO: ✓ Parsed compact capabilities:
   INFO:    Device: v01 (valve_ctrl)
   INFO:    Firmware: 1.0.0
   INFO:    Sensors: ["a", "b", "c"]
   INFO:    Actuators: ["valve"]
   INFO: ✓ Device v01 registered successfully
   ```

**✓ Phase 4 Complete:** Device discovered and registered

---

### Phase 5: End-to-End Testing (10 min)

1. **Navigate to valve module in UI:**
    - Should see "Valve Control" module
    - Toggle switch should show "CLOSED" in red

2. **Test valve toggle:**
    - Press Enter or Space
    - Watch Arduino Serial Monitor:
      ```
      RX: {"p":"ssp/1.0","t":"cmd","i":"v01","d":{"action":"open"}}
      >>> SSP CMD: OPEN
      🔓 Opening valve...
      ✓ Valve OPEN
      ```
    - Servo should move to 90°
    - UI should update to "OPEN" in green

3. **Verify telemetry updates:**
    - Watch Survon logs:
      ```
      INFO: 📦 Chunk #1: {"p":"ssp/1.0","t":"tel",...
      INFO: ✅ COMPLETE MESSAGE
      INFO: Valve telemetry: open=true, position=95.0%
      ```
    - UI should show updated state

4. **Test closing:**
    - Press Enter again
    - Servo should move back to 0°
    - UI should show "CLOSED" in red

5. **Verify continuous telemetry:**
    - Should receive updates every 2 seconds
    - Check Arduino Serial Monitor for TX messages
    - Check Survon logs for parsed telemetry

**✓ Phase 5 Complete:** Full system working!

---

## Troubleshooting

### Problem: Arduino doesn't advertise

**Symptoms:**
```
Bluefruit initialized
(hangs here)
```

**Solutions:**
1. Check shield is fully seated on Arduino
2. Verify SPI pins (CS=8, IRQ=7, RST=4)
3. Try `ble.factoryReset()` in setup
4. Check shield jumpers (SPI mode, not UART)

---

### Problem: Servo jitters or doesn't move

**Symptoms:**
- Servo shakes at position
- Weak or no movement
- Arduino resets when servo moves

**Solutions:**
1. **Jittering:** Add 100µF capacitor across servo power pins
2. **Weak movement:**
    - Use external power for servo (not Arduino 5V)
    - Upgrade to MG996R servo
3. **Arduino resets:** Definitely need external power

---

### Problem: BLE device not found

**Symptoms:**
```
INFO: Scanning for Survon field units...
(no devices found)
```

**Solutions:**
1. Check Arduino Serial Monitor shows "Advertising"
2. Verify device name: `AT+GAPDEVNAME=Survon Valve`
3. Check RSSI (should be > -80 dBm for good connection)
4. Restart Arduino
5. Check BLE adapter on hub: `hcitool dev`

---

### Problem: Commands not received

**Symptoms:**
- UI changes but servo doesn't move
- Arduino Serial shows no RX messages

**Solutions:**
1. **Check device_id matches:**
    - Arduino: `const char* DEVICE_ID = "v01";`
    - config.yml: `device_id: "v01"`
    - config.yml: `bus_topic: "v01"`
2. **Verify handler is registered:**
    - Add log in `initialize_module_handlers`
    - Check Survon startup logs
3. **Check message format:**
    - Add logging in handler's `toggle_valve()`
    - Verify JSON is correct SSP compact format

---

### Problem: Telemetry not updating UI

**Symptoms:**
- Arduino sends telemetry (see Serial Monitor)
- Survon receives messages (see logs)
- UI doesn't change

**Solutions:**
1. **Check topic subscription:**
    - Handler should subscribe to bus_topic
    - Verify in `start_telemetry_listener()`
2. **Check binding updates:**
    - Handler should update module bindings in `update_bindings()`
    - Verify "state" binding is set from "a" value
3. **Check key mapping:**
    - Telemetry sends "a" key
    - Handler reads "a" from "d" object
    - Handler updates "state" binding

---

### Problem: Registration fails

**Symptoms:**
```
INFO: Waiting for device capabilities response...
ERROR: Timeout waiting for device registration response
```

**Solutions:**
1. **Verify capabilities message:**
    - Type "CAP" in Arduino Serial Monitor
    - Should see compact SSP response
    - Verify format matches expected
2. **Check notification stream:**
    - Logs should show "✓ Got notification stream"
    - If not, BLE connection issue
3. **Manual registration:**
    - Trust device first
    - Wait 10 seconds
    - Type "CAP" in Serial Monitor to trigger

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Command latency | 50-100ms (BLE) |
| Telemetry interval | 2000ms |
| Servo transition time | 1500ms (0° to 90°) |
| Message size | 68 bytes (telemetry) |
| RAM usage (Arduino) | ~1200 bytes free |
| Power (SG90) | 5V @ 500mA |
| Power (MG996R) | 6V @ 2A |

---

## Message Flow Diagram

```
┌─────────────┐                 ┌──────────────┐                ┌──────────┐
│   Survon    │                 │   Arduino    │                │  Valve   │
│     UI      │                 │  + Servo     │                │          │
└──────┬──────┘                 └──────┬───────┘                └─────┬────┘
       │                               │                              │
       │ User presses Enter            │                              │
       ├──────────────────────────────►│                              │
       │ {"p":"ssp/1.0","t":"cmd",...} │                              │
       │                               │                              │
       │                               ├─────────────────────────────►│
       │                               │ Move servo 0° → 90°          │
       │                               │                              │
       │                               │◄─────────────────────────────┤
       │                               │ Physical feedback             │
       │                               │                              │
       │◄──────────────────────────────┤                              │
       │ {"p":"ssp/1.0","t":"tel",...} │                              │
       │ {"d":{"a":1,"b":100,"c":5}}   │                              │
       │                               │                              │
       │ UI updates to OPEN (green)    │                              │
       │                               │                              │
       │         (every 2 seconds)     │                              │
       │◄──────────────────────────────┤                              │
       │ Continuous telemetry          │                              │
```

---

## Next Steps

Once basic control is working:

1. **Add position feedback:**
    - Install hall effect sensor
    - Verify actual valve position
    - Detect mechanical failures

2. **Safety features:**
    - Auto-close timeout
    - Maximum open time
    - Over-current detection

3. **Flow measurement:**
    - Add flow sensor
    - Calculate flow rate
    - Display in UI

4. **Multiple valves:**
    - Add more devices (v02, v03, etc.)
    - Zone control
    - Sequential operation

5. **Scheduling:**
    - Timer-based control
    - Conditional logic
    - Integration with other modules
