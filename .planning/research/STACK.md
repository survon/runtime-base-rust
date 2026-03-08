# Stack Research: Council Member System

## Recommended Stack

### Runtime Base (Rust TUI)
- **Language**: Rust 1.75+ (stable)
- **Framework**: Ratatui 0.30+ for terminal UI
- **Message Bus**: Tokio-based async message passing
- **Serialization**: serde for JSON/Binary protocols
- **Logging**: env_logger with structured logging

### Council Member Services (Separate)
- **Language**: Rust (for performance) or Python (for LLM libraries)
- **Runtime**: systemd services on Raspberry Pi
- **Communication**: REST API or gRPC over local network
- **Model Loading**: ONNX Runtime for lightweight models

### LLM Integration
- **Model**: Phi-3 Mini (2.2B parameters) or similar lightweight models
- **Framework**: llama.cpp bindings or huggingface transformers
- **Context**: Limited to 4K-8K tokens for memory constraints
- **Optimization**: Quantization (Q4_K_M) for reduced memory

### Knowledge Retrieval
- **Storage**: SQLite for metadata, file system for documents
- **Indexing**: Full-text search with tantivy or similar
- **Embedding**: Sentence-BERT for semantic search (if memory permits)
- **Caching**: In-memory LRU cache for recent queries

## Rationale

**Rust Runtime**: Existing codebase, proven performance, memory safety
**Separate Services**: Isolation of heavy processing, allows dedicated hardware
**Lightweight Models**: Balance between capability and resource constraints
**Knowledge Prioritization**: Reduce LLM complexity by focusing on retrieval

## What NOT to Use

- **Cloud APIs**: Violates offline requirement
- **Heavy Models (Llama 2 70B)**: Too resource-intensive
- **GPU Acceleration**: Not available on most Raspberry Pi models
- **Docker Containers**: Adds overhead, unnecessary complexity

## Confidence Levels

- **Rust Runtime**: High (existing codebase)
- **Lightweight LLMs**: Medium-High (proven on similar hardware)
- **Separate Services**: Medium (architectural complexity)
- **Knowledge Retrieval**: High (well-established patterns)