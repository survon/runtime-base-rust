# Survon System

## What This Is

An offline, off-grid survival system designed for resilience in challenging scenarios where reliability, modularity, and adaptability are crucial. Survon puts humanity and freedom first, fostering societal resilience and safeguarding knowledge.

### Runtime Architecture
Survon consists of a **modular ecosystem** with three main runtime components:

1. **`runtime-base-rust`** - Core runtime (TUI application)
2. **`runtime-field-rust`** - Portable field runtime  
3. **`survon-os`** - Minimal bash-driven bootable system for Raspberry Pi

## Core Value

A reliable offline system that enables survival resilience through real-time monitoring, control, intelligent guidance, and knowledge preservation without cloud dependencies.

## Requirements

### Validated

- ✓ Modular IoT hub architecture with message bus routing
- ✓ Hardware integration via BLE, radio, USB, and serial connections
- ✓ Knowledge data ingestion from PDFs and text files
- ✓ Basic LLM integration with lightweight models
- ✓ Raspberry Pi deployment with solar power considerations
- ✓ Real-time TUI interface with modular screens
- ✓ Message bus architecture for event routing
- ✓ Transport manager for serial/USB/Bluetooth communication

### Active

- [ ] Council member chat interface with domain-specific advice
- [ ] Lightweight LLM integration with context-aware responses
- [ ] Separate council member service architecture
- [ ] Environmental variable configuration for model selection
- [ ] Reference linking to source documentation
- [ ] Enhanced knowledge retrieval with semantic search
- [ ] Multi-member council support with expertise routing
- [ ] Performance optimization for battery-conscious operation

### Out of Scope

- Heavy AI model training (cost and hardware constraints)
- Real-time video or voice interactions
- Cloud-based knowledge storage
- Multi-user concurrent access
- Mobile app interface
- GPU acceleration (not available on most Raspberry Pi models)
- Docker container deployment (adds overhead)

## Context

Survon is built as a modular IoT hub that intercepts sensor data and device commands from various sources—USB peripherals, Bluetooth devices, radio modules, and custom IoT hardware. The system presents them through a real-time TUI interface.

Current capabilities include:
- Remote gate control via Arduino-based controllers sending open/close commands via radio
- Environmental monitoring with pressure sensors, temperature gauges, and water flow meters
- Equipment status monitoring for well pumps, generators, and greenhouse automation
- Message bus architecture routing events between hardware modules and UI components
- Compatible hardware can be built using Survon protocol adapters on microcontrollers like Arduino, ESP32, or similar platforms
- Devices broadcast sensor readings and accept commands via radio (LoRa, 433MHz), Bluetooth, or direct serial connection to Raspberry Pi hub

All sensor data, control commands, and system events flow through the central message bus, enabling real-time monitoring and control of homestead infrastructure from a single interface.

## Constraints

- **Hardware**: Raspberry Pi 3B+ or newer, solar-powered, limited CPU/RAM (512MB-4GB)
- **Power**: Battery-conscious operation, must work on solar power
- **Cost**: Affordable for homestead deployment ($50-100 per unit)
- **Offline**: No cloud dependencies, must work without internet
- **Real-time**: Sub-second response for critical operations
- **Modularity**: Must support plug-and-play hardware modules
- **Reliability**: 99%+ uptime for essential functions
- **Security**: Local-only data, no external exposure

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust TUI application | Memory safety, performance, existing codebase | ✓ Good |
| Message bus architecture | Decouples hardware from UI, enables modularity | ✓ Good |
| Raspberry Pi deployment | Low cost, low power, proven hardware | ✓ Good |
| Knowledge data ingestion | Enables offline intelligent guidance | ✓ Good |
| Lightweight LLM models | Balance capability with resource constraints | — Pending |
| Council member services | Offload heavy processing, allow dedicated hardware | — Pending |
| Separate OS installer | Simplifies deployment for non-technical users | ✓ Good |
| Solar power focus | Enables true off-grid operation | ✓ Good |

## Council System Context

The council system is **Phase 4** of a 7-phase roadmap, representing the **pivotal middle phase** that transforms Survon from:
- **Basic functionality** → **Intelligent guidance**
- **Hardware control** → **Knowledge-based decision support**
- **Simple queries** → **Domain-specific expertise**

## Council System Architecture

The council system provides domain-specific advice through a service discovery architecture:
- **Separate service architecture** - Council members run as independent processes
- **Service discovery** - Uses mDNS to find `survon-*.local` services
- **REST/gRPC communication** - For client-server interaction
- **Knowledge integration** - Connects to document storage and retrieval

### Council Member Types

Based on the roadmap, council members provide **domain-specific expertise** in areas like:
- Homestead management
- Survival techniques
- Technical troubleshooting
- Agricultural advice
- Emergency response

---
*Last updated: 2026-03-01 after comprehensive planning*