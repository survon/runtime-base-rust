# Council System Documentation

## Overview

The Council system is a multi-member advisory system that provides intelligent guidance and decision support for the Survon smart homestead. It routes queries to appropriate advisors based on their expertise and builds consensus from multiple responses.

## Features

- **Multi-member Support**: Multiple council members with different expertise areas
- **Expertise Routing**: Queries automatically routed to appropriate advisors
- **Consensus Building**: Builds consensus from multiple council member responses
- **Real-time Communication**: Message-based communication with advisors
- **CLI Integration**: Command-line interface for council management
- **UI Interface**: Interactive chat interface for council interaction

## Getting Started

### Prerequisites

- Survon runtime v1.0 or later
- Council modules properly configured
- Network connectivity to council advisors

### Basic Usage

#### Via CLI

```bash
# Start council chat
./target/release/runtime-base-rust council chat --advisor hardware_expert --query "What is the current system status?"

# Check council status
./target/release/runtime-base-rust council status

# List available advisors
./target/release/runtime-base-rust council advisors list

# Run council tests
./target/release/runtime-base-rust council test --all
```

#### Via UI

1. Start the Survon application: `./target/release/runtime-base-rust`
2. Press `Tab` to navigate to council interface
3. Use arrow keys to select advisors
4. Press `Enter` to send messages
5. Use `Esc` to return to overview

## Council Advisors

### Available Advisors

| Advisor | Expertise | Description |
|---------|-----------|-------------|
| hardware_expert | Device management | Hardware troubleshooting and device management |
| knowledge_base | Information retrieval | Knowledge base queries and documentation |
| system_monitor | System health | System performance and health monitoring |
| security_specialist | Security | Security and access control specialist |

### Adding Advisors

#### Via CLI

```bash
./target/release/runtime-base-rust council advisors add \
  --name "network_expert" \
  --expertise "networking,security" \
  --device_id "device_12"
```

#### Via Configuration

Add to your council configuration:

```yaml
advisors:
  - name: network_expert
    expertise: networking,security
    device_id: device_12
    online: true
```

## Commands Reference

### Council Chat

```bash
./target/release/runtime-base-rust council chat [OPTIONS]
```

Options:
- `--advisor NAME` - Specific advisor to chat with
- `--query TEXT` - Query to send to council

### Council Status

```bash
./target/release/runtime-base-rust council status [OPTIONS]
```

Options:
- `--detailed` - Show detailed advisor information

### Advisor Management

```bash
./target/release/runtime-base-rust council advisors SUBCOMMAND
```

Subcommands:
- `list` - List all available advisors
- `add` - Add new advisor
- `remove` - Remove advisor
- `update` - Update advisor configuration

### Testing

```bash
./target/release/runtime-base-rust council test [OPTIONS]
```

Options:
- `--all` - Run all council tests
- `--test-type TYPE` - Run specific test type (discovery, multi-member, routing, consensus, commands, status)

## Configuration

### Council Configuration

Council configuration is stored in `~/.survon/council.yml`:

```yaml
council:
  enabled: true
  advisors:
    - name: hardware_expert
      expertise: hardware,devices,troubleshooting
      device_id: device_1
      online: true
      priority: high
    - name: knowledge_base
      expertise: knowledge,documentation,research
      device_id: device_2
      online: true
      priority: normal
  consensus_strategy: weighted_average
  response_timeout: 5000
  max_advisors: 10
```

### Advisor Configuration

Each advisor can have specific configuration:

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
    preferences:
      response_format: json
      timeout: 3000
```

## Troubleshooting

### Common Issues

#### Advisors Not Responding

1. Check advisor status: `survon council status`
2. Verify device connectivity
3. Check advisor configuration
4. Restart council service

#### CLI Commands Not Working

1. Verify council module is enabled
2. Check permissions
3. Verify network connectivity
4. Check logs for errors

#### UI Interface Not Available

1. Verify council module is loaded
2. Check keyboard focus
3. Verify screen rendering
4. Check for conflicts with other modules

### Debug Logging

Enable debug logging for council system:

```bash
DEBUG=true ./target/release/runtime-base-rust
```

### Logs Location

Council logs are stored in:
- `./logs/council/error.log`
- `./logs/council/warn.log`
- `./logs/council/info.log`
- `./logs/council/debug.log`

## Best Practices

### Query Routing

- Use specific advisors for domain-specific questions
- Allow general queries for broad topics
- Consider advisor expertise when formulating queries
- Use consensus for complex decisions

### Performance

- Monitor advisor response times
- Set appropriate timeouts
- Limit concurrent queries
- Use caching for frequent queries

### Security

- Verify advisor identities
- Use encrypted communication
- Monitor advisor activity
- Implement access controls

## API Reference

### Council Service

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

## Examples

### Simple Query

```bash
./target/release/runtime-base-rust council chat --query "What is the current temperature?"
```

### Expert Consultation

```bash
./target/release/runtime-base-rust council chat --advisor hardware_expert --query "Why is device_3 not responding?"
```

### System Status

```bash
./target/release/runtime-base-rust council status --detailed
```

### Testing

```bash
./target/release/runtime-base-rust council test --all
```

## Contributing

### Adding New Advisors

1. Implement advisor logic in `src/module/strategies/council/`
2. Add advisor to configuration
3. Update expertise routing
4. Add tests for new advisor
5. Update documentation

### Improving Consensus

1. Implement new consensus strategies
2. Add configuration options
3. Update routing logic
4. Add performance metrics
5. Update documentation

## Support

For support and questions:

- Check the troubleshooting section
- Review the logs
- Enable debug logging
- Check the Survon community forums
- Review the source code

## Version History

- v1.0 - Initial council system implementation
- v1.1 - Added multi-member support and expertise routing
- v1.2 - Added CLI commands and UI interface
- v1.3 - Added comprehensive testing and documentation
- v1.4 - Enhanced consensus building and performance

## License

This documentation is licensed under the MIT License.