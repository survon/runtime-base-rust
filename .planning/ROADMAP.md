# Roadmap: Survon Project

**Created:** 2026-03-01
**Phases:** 7
**Requirements:** 14 v1 requirements mapped
**Coverage:** 100% ✓

## System Context

Survon is an **offline, off-grid survival system** designed for resilience in challenging scenarios where reliability, modularity, and adaptability are crucial. It puts humanity and freedom first, fostering societal resilience and safeguarding knowledge.

### Runtime Architecture
Survon consists of a **modular ecosystem** with three main runtime components:

1. **`runtime-base-rust`** - Core runtime (TUI application)
2. **`runtime-field-rust`** - Portable field runtime  
3. **`survon-os`** - Minimal bash-driven bootable system for Raspberry Pi

### Council System Positioning

The council system is **Phase 4** of 7 phases, representing the **pivotal middle phase** that transforms Survon from:
- **Basic functionality** → **Intelligent guidance**
- **Hardware control** → **Knowledge-based decision support**
- **Simple queries** → **Domain-specific expertise**

### Phase Dependencies
```
Phase 1: Foundation → Phase 2: Knowledge Base → Phase 3: Basic LLM
    ↓
Phase 4: Council System (requires Phase 3)
    ↓
Phase 5: Enhanced Knowledge (requires Phase 4)
    ↓
Phase 6: Multi-Member Council (requires Phase 5)
    ↓
Phase 7: Optimization
```

## Phase Structure

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 1 | Foundation | Basic IoT hub functionality | FOUND-01,FOUND-02,FOUND-03,FOUND-04,FOUND-05 | Hardware modules communicate successfully |
| 2 | Knowledge Base | Document ingestion and search | KNOW-01,KNOw-02,KNOw-03,KNOw-04,KNOw-05 | PDF/text files indexed and searchable |
| 3 | Basic LLM | Lightweight model integration | LLM-01,LLM-02,LLM-03,LLM-04,LLM-05 | Model loads and responds to queries |
| 4 | Council System | Domain-specific advice interface | COUNCIL-01,COUNCIL-02,COUNCIL-03,COUNCIL-04,COUNCIL-05 | User can chat with council member |
| 5 | Enhanced Knowledge | Semantic search and context | ENH-01,ENH-02,ENH-03,ENH-04,ENH-05 | Context-aware responses with references |
| 6 | Multi-Member | Council coordination and routing | MM-01,MM-02,MM-03,MM-04,MM-05 | Multiple members with expertise routing |
| 7 | Optimization | Performance and reliability | OPT-01,OPT-02,OPT-03,OPT-04,OPT-05 | Sub-10s response times, 99% uptime |

## Phase Details

### Phase 1: Foundation
**Goal:** Basic IoT hub functionality
**Requirements:** FOUND-01,FOUND-02,FOUND-03,FOUND-04,FOUND-05
**Success criteria:**
1. Service starts within 5 seconds
2. Hardware modules (BLE, USB, radio) communicate successfully
3. Message bus routes events between modules
4. Transport manager handles serial/USB/Bluetooth
5. Basic UI displays connected devices

### Phase 2: Knowledge Base
**Goal:** Document ingestion and search
**Requirements:** KNOW-01,KNOw-02,KNOw-03,KNOw-04,KNOw-05
**Success criteria:**
1. PDF/text files ingested successfully
2. Full-text search returns relevant results
3. Document metadata indexed
4. Knowledge base accessible to other components
5. Search performance under 2 seconds

### Phase 3: Basic LLM
**Goal:** Lightweight model integration
**Requirements:** LLM-01,LLM-02,LLM-03,LLM-04,LLM-05
**Success criteria:**
1. Model loads from environment variables
2. Simple queries return responses under 5 seconds
3. Memory usage stays under 500MB
4. Model handles basic conversation
5. Error handling for model failures

### Phase 4: Council System
**Goal:** Domain-specific advice interface
**Requirements:** COUNCIL-01,COUNCIL-02,COUNCIL-03,COUNCIL-04,COUNCIL-05
**Success criteria:**
1. Council member service runs as separate process
2. User can start conversation with council member
3. Typing indicators display during processing
4. Conversation history maintained
5. Basic domain-specific responses provided

### Phase 5: Enhanced Knowledge
**Goal:** Semantic search and context
**Requirements:** ENH-01,ENH-02,ENH-03,ENH-04,ENH-05
**Success criteria:**
1. Semantic search with embeddings
2. Context injection into LLM prompts
3. Reference linking to source documents
4. Citation display in responses
5. Context-aware responses

### Phase 6: Multi-Member
**Goal:** Council coordination and routing
**Requirements:** MM-01,MM-02,MM-03,MM-04,MM-05
**Success criteria:**
1. Multiple council members supported
2. Expertise routing to appropriate member
3. Consensus building for related queries
4. Conflict resolution for contradictory advice
5. Council member discovery and registration

### Phase 7: Optimization
**Goal:** Performance and reliability
**Requirements:** OPT-01,OPT-02,OPT-03,OPT-04,OPT-05
**Success criteria:**
1. Sub-10 second response times consistently
2. 99%+ uptime availability
3. Memory usage under 1GB total
4. Battery usage optimized for solar power
5. Graceful degradation for failures

## Success Criteria

### Phase 1 Success
- Service starts within 5 seconds
- Hardware modules connect and communicate
- Message bus routes events correctly
- Basic UI displays device status

### Phase 2 Success
- PDF/text ingestion completes in under 30 seconds
- Full-text search returns relevant results
- Document metadata searchable
- Knowledge base accessible to other components

### Phase 3 Success
- Model loads within 10 seconds
- Simple query returns response under 5 seconds
- Memory usage stays under 500MB
- Basic conversation capability

### Phase 4 Success
- Council member service starts successfully
- User can start and maintain conversation
- Typing indicators work during processing
- Basic domain-specific advice provided

### Phase 5 Success
- Semantic search returns relevant context
- Context injection improves response quality
- Reference links point to correct sources
- Citation display is accurate

### Phase 6 Success
- Multiple council members discoverable
- Queries routed to appropriate expertise
- Consensus building works for related topics
- Conflict resolution handles contradictions

### Phase 7 Success
- Response times consistently under 10 seconds
- System available 99%+ of the time
- Memory usage optimized for target hardware
- Battery usage suitable for solar power

## Dependencies

### Phase Dependencies
- Phase 2 requires Phase 1 (hardware communication)
- Phase 3 requires Phase 2 (knowledge base)
- Phase 4 requires Phase 3 (LLM integration)
- Phase 5 requires Phase 4 (council system)
- Phase 6 requires Phase 5 (enhanced knowledge)
- Phase 7 requires all previous phases

### Technical Dependencies
- Hardware communication requires correct drivers
- Knowledge base requires document ingestion
- LLM integration requires model availability
- Council system requires working service communication
- Semantic search requires indexed knowledge base
- Multi-member support requires council system working

## Risk Mitigation

### High-Risk Items
- **Hardware Compatibility**: Different devices may have varying protocols
- **Model Performance**: Lightweight models may not be sufficiently capable
- **Memory Constraints**: Multiple services may exceed hardware limits
- **Response Time**: Complex queries may exceed acceptable limits

### Mitigation Strategies
- **Hardware Abstraction**: Use standardized protocols where possible
- **Progressive Enhancement**: Start simple, add complexity gradually
- **Resource Monitoring**: Track memory/CPU usage in real-time
- **Performance Targets**: Set and enforce response time limits

## Files: `.planning/ROADMAP.md`, `.planning/STATE.md`