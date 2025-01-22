# Survon Base Runtime

The Survon Base Runtime is the foundation of the Survon system, managing module registration, event orchestration, 
and communication between modules. It ensures modularity, scalability, and reliability in off-grid, survival-focused scenarios.

* *NOTE: This first runtime is built in `rust`. Depending on how well this goes with power consumption and community adoption, etc., more 
runtimes may become available.*

---
## Project Structure
```text 
survon-base-rust/
├── Cargo.toml
├── src/
│   ├── main.rs             # Core runtime: Manages modules, system updates, and orchestration
│   ├── event_bus.rs        # Event Bus: Manages event publication and subscription
│   ├── orchestrator.rs     # Orchestrator: Coordinates communication between modules
│   ├── tests/
│   │   └── integration.rs  # Integration tests for the runtime
└── modules/
    └── (empty initially)
```
## Categories and Events

### Event Categories
Modules are grouped into categories, each with predefined, utility-focused events. These events scale universally and ensure consistency across modules.

#### **Communication (`com`)**
- **Core Events**:
    - `messageReceived`
    - `messageSent`
    - `connectionEstablished`
    - `connectionLost`
- **Utility Events**:
    - `marco` (ping sent to all modules in this category)
    - `diagnostics` (check module status, e.g., connection health)

#### **Monitoring (`mon`)**
- **Core Events**:
    - `dataThresholdExceeded` (e.g., temperature exceeds limit)
    - `dataThresholdRecovered` (e.g., temperature returns to normal)
    - `dataStale` (e.g., sensor data has not been updated for a while)
- **Utility Events**:
    - `marco` (ping for module response)
    - `diagnostics` (verify all monitored sources are reporting correctly)

#### **Sensors (`sen`)**
- **Core Events**:
    - `sensorConnected`
    - `sensorDisconnected`
    - `sensorDataAvailable`
    - `sensorFailure`
- **Utility Events**:
    - `marco` (respond uniquely for each sensor, e.g., emit a test signal)
    - `diagnostics` (verify sensor health and calibration)

#### **Power (`pow`)**
- **Core Events**:
    - `powerLow`
    - `powerRestored`
    - `powerShutdown`
    - `powerStartup`
- **Utility Events**:
    - `marco` (respond with power-specific acknowledgment, e.g., blink power LED)
    - `diagnostics` (run checks for power supply status and battery health)

#### **Notifications (`not`)**
- **Core Events**:
    - `systemNotification` (generic system broadcast)
    - `userAlert` (user-specific alert or prompt)
- **Utility Events**:
    - `marco` (broadcast notification acknowledgment)
    - `diagnostics` (test notification delivery mechanisms)

---

## Event Signature

The Survon system uses a unified event structure for communication and orchestration. Here’s the event signature:

```rust
/// Represents an event in the Survon system.
#[derive(Debug, Clone)]
pub struct Event {
    pub category: String,           // E.g., "com", "mon", "sen"
    pub event_type: String,         // E.g., "marco", "diagnostics"
    pub payload: serde_json::Value, // Event-specific data
    pub meta: EventMeta,            // Metadata for cross-module communication
}

/// Metadata for cross-module communication and tracking.
#[derive(Debug, Clone)]
pub struct EventMeta {
    pub source: String, // Module that generated the event
    pub author: String, // Module's author identifier
    pub timestamp: u64, // Event creation timestamp (e.g., UNIX time)
}
```

---

## Code Snippets

### Sending a `marco` Event
```rust
use serde_json::json;

fn send_marco_event() -> Event {
    Event {
        category: "com".to_string(),
        event_type: "marco".to_string(),
        payload: json!({}), // No specific data needed for a ping
        meta: EventMeta {
            source: "orchestrator".to_string(),
            author: "survon_core".to_string(),
            timestamp: 1672531200,
        },
    }
}
```

### Responding to a `marco` Event
```rust
fn on_marco(event: &Event) -> Event {
    if event.event_type == "marco" {
        Event {
            category: event.category.clone(),
            event_type: "marcoResponse".to_string(),
            payload: serde_json::json!({
                "response": "Polo",
                "module": "irc"
            }),
            meta: EventMeta {
                source: "irc_module".to_string(),
                author: "example_author".to_string(),
                timestamp: 1672531210,
            },
        }
    } else {
        panic!("Unexpected event type: {}", event.event_type);
    }
}
```

---

## Installation Instructions

1. **Clone the Repository**
   ```bash
   git clone https://github.com/survon/runtime-base-rust.git
   cd runtime-base-rust
   ```

2. **Build the Project**
   ```bash
   cargo build
   ```

3. **Run the Base Runtime**
   ```bash
   cargo run
   ```

4. **Run Tests**
   ```bash
   cargo test
   ```

---

## Contribution Guidelines

We welcome community contributions! Please adhere to the following:
- Follow the [Module Packaging Standards](#module-packaging-standards) when creating modules.
- Ensure all modules use the predefined event categories and comply with the event signature.

For detailed guidelines, refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file in this repository.

---

Let me know if further refinement is needed!
