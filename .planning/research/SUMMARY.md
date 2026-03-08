# Research Summary: Council Member System

## Key Findings

### Stack Recommendations
**Core Technology**: Rust TUI application with Ratatui framework
**Model Choice**: Phi-3 Mini (2.2B parameters) or similar lightweight models
**Architecture**: Separate council member services for heavy processing
**Knowledge Management**: SQLite + file system with full-text search

### Table Stakes Features
- Terminal-based chat interface with conversation history
- Document search across ingested PDFs/text files
- Context injection into LLM prompts
- Reference linking to source documentation
- Basic model management (selection, temperature, context)

### Critical Success Factors
- **Model Selection**: Balance between capability and resource constraints
- **Knowledge Prioritization**: Focus on retrieval over complex reasoning
- **Performance Targets**: Sub-10-second response times
- **Safety Considerations**: No dangerous medical/legal advice

### Watch Out For
- **Resource Exhaustion**: Memory usage above 80% of available RAM
- **Hallucinations**: Fabricated information presented as fact
- **Slow Responses**: User abandonment due to delays
- **Over-Engineering**: Too large a model for hardware constraints

## Recommended Implementation Order

1. **Phase 1**: Basic service communication and document storage
2. **Phase 2**: Lightweight LLM integration and basic chat
3. **Phase 3**: Knowledge retrieval and context injection
4. **Phase 4**: Response processing and reference linking
5. **Phase 5**: Multi-member support and council coordination

## Confidence Levels

- **Rust Runtime**: High (existing codebase)
- **Lightweight LLMs**: Medium-High (proven on similar hardware)
- **Separate Services**: Medium (architectural complexity)
- **Knowledge Retrieval**: High (well-established patterns)

## Files: `.planning/research/`