# Council System Architecture

## Overview

The Council system is a multi-member advisory system that provides intelligent guidance and decision support for the Survon smart homestead. It routes queries to appropriate advisors based on their expertise and builds consensus from multiple responses.

## Architecture Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                    Council System Architecture                    │
├────────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    Council UI                          │ │
│  │  ┌────────────────────────────────────────────────────────────────┐ │
│  │  │  Chat Interface  │  │  Advisor List    │  │  │
│  │  │  (Message History)│  │  (Expertise)      │  │  │
│  │  └────────────────────────────────────────────────────────────────┘ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                            │
├────────────────────────────────────────────────────────────────┤
│  CLI Interface ←→ Message Bus ←→ Council Service      │
│  (Commands)       (BusMessage)      (CouncilHandler)    │
│                                                            │
├────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │        Database                          │ │
│  │  ┌────────────────────────────────────────────────────────────────┐ │
│  │  │  Advisor Config    │  │  Knowledge Base   │  │
│  │  │  (Expertise)       │  │  (Queries)        │  │
│  │  └────────────────────────────────────────────────────────────────┘ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                            │
└────────────────────────────────────────────────────────────────┘
```

## Components

### Council Service

**Location**: `src/module/strategies/council.rs`

**Responsibilities**:
- Handle council messages and routing
- Manage advisor communication
- Build consensus from multiple responses
- Handle council commands

### Council Handler

**Location**: `src/module/strategies/council.rs`

**Responsibilities**:
- Process incoming council messages
- Route queries to appropriate advisors
- Handle advisor responses
- Build consensus
- Manage council state

### Message Bus

**Location**: `src/util/io/bus/`

**Responsibilities**:
- Route messages between components
- Handle asynchronous communication
- Manage message persistence
- Support topic-based routing

### Database

**Location**: `src/util/database/`

**Responsibilities**:
- Store advisor configurations
- Persist knowledge base
- Store council state
- Handle advisor status

## Data Flow

### Query Processing

1. **User Input**: User submits query via CLI or UI
2. **Message Routing**: Message routed to council service via message bus
3. **Expertise Analysis**: Query analyzed for expertise requirements
4. **Advisor Selection**: Appropriate advisors selected based on expertise
5. **Query Distribution**: Query distributed to selected advisors
6. **Response Collection**: Responses collected from advisors
7. **Consensus Building**: Consensus built from multiple responses
8. **Response Delivery**: Final response delivered to user

### Command Processing

1. **User Command**: User submits command via CLI or UI
2. **Command Routing**: Command routed to appropriate advisor
3. **Command Execution**: Command executed by advisor
4. **Result Collection**: Execution results collected
5. **Response Delivery**: Results delivered to user

## Communication Protocols

### Message Format

```json
{
  "topic": "council.query",
  "query": "device_status",
  "advisor": "hardware_expert",
  "timestamp": 1234567890,
  "source": "survon_tui"
}
```

### Response Format

```json
{
  "topic": "council.response",
  "advisor": "hardware_expert",
  "response": "device_status",
  "status": {"online": true, "uptime": 12345},
  "timestamp": 1234567890,
  "source": "hardware_expert"
}
```

## Configuration

### Council Configuration

```yaml
council:
  enabled: true
  consensus_strategy: weighted_average
  response_timeout: 5000
  max_advisors: 10
  expertise_routing: true
```

### Advisor Configuration

```yaml
advisors:
  - name: hardware_expert
    expertise: hardware,devices,troubleshooting
    device_id: device_1
    online: true
    priority: high
    capabilities:
      - device_status
      - reboot
      - firmware_update
      - diagnostics
```

## Testing Architecture

### Test Structure

```
src/tests/council/
├── mod.rs              # Main test module
├── service_discovery.rs # Service discovery tests
├── multi_member.rs     # Multi-member support tests
├── expertise_routing.rs # Expertise routing tests
├── consensus_building.rs # Consensus building tests
├── command_handling.rs  # Command handling tests
└── status_updates.rs    # Status update tests
```

### Test Categories

- **Service Discovery**: Tests advisor discovery and initialization
- **Multi-Member Support**: Tests handling multiple advisors
- **Expertise Routing**: Tests query routing based on expertise
- **Consensus Building**: Tests consensus building from multiple responses
- **Command Handling**: Tests council command processing
- **Status Updates**: Tests advisor status updates

## Performance Considerations

### Scalability

- Support for up to 100+ advisors
- Efficient query routing algorithms
- Asynchronous response handling
- Load balancing across advisors

### Response Times

- Query routing: <100ms
- Consensus building: <500ms
- Command execution: <1s
- Status updates: <50ms

### Resource Usage

- Memory: ~50MB base + 1MB per advisor
- CPU: Minimal for routing, moderate for consensus building
- Network: Optimized message batching
- Storage: Efficient database indexing

## Security Considerations

### Authentication

- Advisor identity verification
- Secure communication channels
- Access control lists
- Audit logging

### Data Protection

- Encrypted storage for sensitive data
- Secure knowledge base
- Privacy controls for user queries
- Data retention policies

### Integrity

- Message integrity verification
- Response validation
- Error handling and recovery
- Consistency guarantees

## Integration Points

### With Other Modules

- **Message Bus**: Primary communication channel
- **Database**: Configuration and state storage
- **UI Framework**: User interface integration
- **CLI System**: Command-line interface
- **Discovery Manager**: Advisor discovery

### External Systems

- **Hardware Devices**: Advisor endpoints
- **Knowledge Bases**: Information sources
- **Monitoring Systems**: Status reporting
- **Security Systems**: Access control

## Future Enhancements

### Planned Features

- **Machine Learning**: Adaptive expertise routing
- **Advanced Consensus**: Multi-factor consensus algorithms
- **Predictive Analytics**: Proactive advisor suggestions
- **Enhanced Security**: Zero-trust architecture

### Performance Improvements

- **Caching**: Intelligent response caching
- **Parallel Processing**: Concurrent advisor queries
- **Optimization**: Query routing optimization
- **Monitoring**: Real-time performance metrics

## API Reference

### Service Interface

```rust
use crate::module::strategies::council::CouncilHandler;

async fn query_council(handler: &CouncilHandler, query: &str, advisor: &str) -> Result<String, color_eyre::Report> {
    // Query council system
    let response = handler.handle_council_message(query).await?;
    Ok(response)
}
```

### Message Bus Integration

```rust
use crate::util::io::bus::MessageBus;

async fn publish_council_message(bus: &MessageBus, topic: &str, payload: &str) -> Result<(), color_eyre::Report> {
    let message = crate::util::io::bus::BusMessage::new(
        topic.to_string(),
        payload.to_string(),
        "survon_tui".to_string(),
    );
    bus.publish(message).await?;
    Ok(())
}
```

## Version History

- v1.0 - Initial council system implementation
- v1.1 - Added multi-member support and expertise routing
- v1.2 - Added CLI commands and UI interface
- v1.3 - Added comprehensive testing and documentation
- v1.4 - Enhanced consensus building and performance

## License

This architecture documentation is licensed under the MIT License.