# Hyper Memory — Implementation Task Breakdown

This document tracks the granular development tasks, milestones, subtasks, and progress for Hyper Memory.

## Workflow Rules & Protocols
1. **Target Platform Compatibility**:
   - Must build and run cleanly across **Windows, macOS, and Linux**.
   - Use cross-platform path manipulation (`PathBuf`), bundled database dependencies (`rusqlite` bundled), and avoid platform-specific shell scripts for core execution.
   - Implement version bumping (`vup`) as a native Rust CLI utility or platform-agnostic runner so it runs identically on Windows (CMD/PowerShell), macOS, and Linux.
2. **Subtask Execution Protocol**:
   - Each subtask must be implemented, thoroughly unit/integration tested, and verified before progressing.
   - Upon completing each subtask:
     - Run automated tests (`cargo test`).
     - Bump version using `vup` (increment patch version in `Cargo.toml`).
     - Record the completed subtask in `docs/changelog.md` with version and details.
     - Commit changes with a clean English message (`git commit -m "..."`) and push to remote (`git push origin <branch>`).
2. **Language Protocol**:
   - Documentation, code comments, tests, and git commit messages are strictly in English.
   - User conversation is in Korean.

---

## Task Matrix & Milestones

### Milestone 1: Core Foundation & Storage Engine
- [x] **Task 1.1: Project Initialization & Cargo Setup**
  - Create Rust workspace / package layout with CLI and Core library.
  - Setup core dependencies (`tokio`, `axum`, `serde`, `blake3`, `rusqlite`, `chrono`, `tracing`, `clap`, etc.).
  - Setup version script (`vup`) and project tooling.
- [x] **Task 1.2: Domain Models & Schema Definitions**
  - Implement models: `Document`, `Chunk`, `Memory`, `Entity`, `Edge`, `UserProfile`.
  - Implement serialization/deserialization with `serde`.
- [x] **Task 1.3: Storage Engine & Embedded Database**
  - SQLite/Redb storage engine implementation with migrations and connection pooling.
  - CRUD operations for Documents, Chunks, Memories, Entities, and Graph Edges.
  - BLAKE3 content hashing and deduplication repository.

### Milestone 2: Text Ingestion & Memory Extraction Pipeline
- [x] **Task 2.1: Content Ingestion, Chunking & Normalization**
  - Text and Markdown normalizer (trimming, Unicode normalization, whitespace collapsing).
  - Chunking strategies (sliding window with overlap, header-aware Markdown chunking).
  - Content dedup & chunk reference linking to avoid duplicate storage.
- [x] **Task 2.2: Memory Extraction & Provider Abstractions**
  - Provider trait definitions: `LLMProvider`, `EmbeddingProvider`.
  - Cheap heuristic rule filter to eliminate conversational noise without LLM overhead.
  - Deterministic/Local/Mock LLM and Embedding provider implementation for local-first testing and execution.
  - Memory extraction pipeline: parsing atomic facts, decisions, entities, relations, confidence, and importance.

### Milestone 3: Temporal Memory Graph & Consolidation
- [x] **Task 3.1: Temporal State Machine & Fact Updating**
  - Fact supersession logic (`is_latest = false`, `valid_until` setting when new contradicting or updated memory arrives).
  - Memory relation graph: `UPDATES`, `EXTENDS`, `DERIVES`, `CONTRADICTS`, `SUPPORTS`.
  - Contradiction resolution and conflict logging.
- [x] **Task 3.2: Memory Lifecycle & Tier Management**
  - Hot/Warm/Cold tier state transitions based on recency, access count, and pinning (`is_pinned`).
  - Access tracking and recency decay for temporary/episodic memories.
- [x] **Task 3.3: Memory Consolidation Engine**
  - Identification of repeated/similar memories for entity consolidation.
  - Canonical memory synthesis with provenance links (`derived_from`).

### Milestone 4: Multi-Engine Retrieval & Search Pipeline
- [x] **Task 4.1: Inverted Index Full-Text Search (FTS)**
  - Embedded FTS (BM25 tokenization, term frequency, document frequency).
  - Fast prefix and term lookup over canonical memories and documents.
- [x] **Task 4.2: Vector Storage & Approximate Nearest Neighbor (ANN)**
  - Vector storage with cosine similarity and Euclidean distance.
  - Quantization support (FP32, FP16, and INT8 simulated dot-product quantization).
  - In-memory vector index with embedding cache.
- [x] **Task 4.3: Query Router, Hybrid Reranker & Context Compiler**
  - Query classification (`Profile`, `Exact`, `Lexical`, `Semantic`, `Temporal`, `Hybrid`).
  - Weighted hybrid scoring: $w_{lex} + w_{sem} + w_{rec} + w_{imp} + w_{conf}$.
  - Graph 1-hop / 2-hop expansion.
  - Context compiler with strict token budgeting ($\le 2000$ tokens) for Agent consumption.

### Milestone 5: Obsidian Vault & Wiki Compiler
- [x] **Task 5.1: Wiki Intermediate Representation (IR) & Vault Generation**
  - Deterministic IR compiler generating entity, decision, and daily note data.
  - Markdown page generator adhering to Obsidian Vault layout (`People/`, `Projects/`, `Technologies/`, `Decisions/`, `Daily/`, `Sources/`).
  - Structured Frontmatter and wikilinks (`[[Page Name]]`).
- [x] **Task 5.2: Strict Section Demarcation & Safe Atomic Writes**
  - Enforcement of `<!-- HYPER_MEMORY:AUTO:BEGIN -->` and `<!-- HYPER_MEMORY:AUTO:END -->`.
  - Preservation of user-authored sections and manual notes.
  - Atomic writing via temporary files to prevent data corruption.
- [x] **Task 5.3: Bidirectional Obsidian Sync & Change Detection**
  - File watcher / diff engine for markdown changes.
  - Control tag parser (`#hypermemory/pin`, `#hypermemory/archive`, `#hypermemory/no-ai`).
  - Semantic diff extraction converting manual note updates into memory update candidates.

### Milestone 6: REST API & MCP Server
- [ ] **Task 6.1: Axum REST API Server**
  - Endpoints:
    - `/v1/documents` (Ingest document)
    - `/v1/memories` (CRUD memories, pin, archive)
    - `/v1/search` (Hybrid search)
    - `/v1/context` (Compact agent context)
    - `/v1/profile/:container` (User profile KV)
    - `/v1/wiki/sync` & `/v1/wiki/rebuild`
    - `/v1/graph/entity/:id` & `/v1/graph/memory/:id`
  - Structured logging, error responses, and CORS support.
- [ ] **Task 6.2: Standard MCP (Model Context Protocol) Server**
  - JSON-RPC 2.0 stdio server compliant with Anthropic MCP specifications.
  - Tools implementation: `memory_add`, `memory_search`, `memory_get`, `memory_forget`, `memory_update`, `profile_get`, `wiki_search`, `wiki_get_page`, `wiki_update_page`, `project_context`, `source_get`.

### Milestone 7: CLI Interface, Benchmarking & Comprehensive Validation
- [ ] **Task 7.1: CLI Implementation (`hyper-memory`)**
  - Subcommands: `start`, `status`, `add`, `search`, `context`, `wiki sync`, `doctor`, `benchmark`.
- [ ] **Task 7.2: Benchmark Suite & Performance Verification**
  - Recall, latency, deduplication rate, memory footprint, and Obsidian preservation benchmarks.
- [ ] **Task 7.3: Documentation & README**
  - Elaborate README showcasing Hyper Memory's greatest strengths (Local-first, Temporal Graph, Obsidian Two-way Sync, Token Budgeting).
  - Final integration testing across REST, MCP, and CLI.
