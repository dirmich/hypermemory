# Hyper Memory — System Architecture & Implementation Plan

## 1. Executive Summary

**Hyper Memory** is a local-first, high-performance AI memory and knowledge engine engineered in Rust. It bridges the critical divide between unstructured agent conversations/documents and structured human-readable knowledge. Instead of treating memory as a pure vector store or transient chat logs, Hyper Memory builds a temporal knowledge graph with automatic fact updates, contradiction detection, and bidirectional synchronization with an Obsidian markdown vault.

This document outlines the end-to-end technical blueprint, domain models, indexing strategies, API designs, and execution milestones.

### 1.1 Target Platforms
- **Windows (x86_64, aarch64)**
- **macOS (Apple Silicon aarch64, Intel x86_64)**
- **Linux (x86_64, aarch64)**

All file paths, atomic write operations, database drivers, and CLI tools are built to run cross-platform without platform-specific external system dependencies (using bundled SQLite, pure Rust BLAKE3, standard `std::path` / `PathBuf`, cross-platform file locking, and tempfile swaps).

---

## 2. Core Architectural Pillars

```
+-----------------------------------------------------------------------+
|                                Clients                                |
|          AI Agents (Claude/Cursor/Hermes) · Obsidian · CLI · Web UI   |
+-----------------------------------+-----------------------------------+
                                    |
                    +---------------+---------------+
                    | REST API (Axum) / MCP Server  |
                    +---------------+---------------+
                                    |
+-----------------------------------v-----------------------------------+
|                         Core Engine (Rust)                            |
|                                                                       |
|  +-------------------+  +--------------------+  +------------------+  |
|  | Ingestion Engine  |  | Memory Extractor   |  | Temporal Graph   |  |
|  | - Normalize       |  | - Heuristic Filter |  | - Updates/Latest |  |
|  | - BLAKE3 Fingerpr |  | - LLM Provider     |  | - Contradictions |  |
|  | - AST/Text Chunk  |  | - Entity Resolver  |  | - Lifecycle/Tier |  |
|  +-------------------+  +--------------------+  +------------------+  |
|                                                                       |
|  +-------------------+  +--------------------+  +------------------+  |
|  | Storage & Cache   |  | Search & Routing   |  | Wiki Compiler    |  |
|  | - Redb / SQLite   |  | - Inverted FTS/BM25|  | - IR Generator   |  |
|  | - Embedding Cache |  | - HNSW Vector ANN  |  | - Vault Writer   |  |
|  | - Content Dedup   |  | - Hybrid Reranker  |  | - AUTO Protected |  |
|  | - Compact CSR Gra |  | - Context Compiler |  | - Obsidian Sync  |  |
|  +-------------------+  +--------------------+  +------------------+  |
+-----------------------------------------------------------------------+
```

### 2.1 The Knowledge & Projection Principle
- **Graph is Source of Truth**: The canonical temporal graph, atomic memories, and entity relationships stored in embedded storage represent the authoritative ground truth.
- **Obsidian is Human Projection**: Obsidian Markdown notes are deterministic projections rendered by the Wiki Compiler. Manual user edits inside user sections are preserved, while modifications inside AUTO sections or user edits trigger semantic diff analysis that proposes or executes memory updates.
- **Zero Token Waste**: Ingestion filters out conversational noise without expensive LLM calls. Queries are routed specifically (Profile KV lookup, Lexical FTS, or Vector ANN) rather than performing multi-modal vector scans on every query.

---

## 3. Component Architecture & Data Schemas

### 3.1 Logical Storage Models

1. **Document**:
   - `id`: Compact String/UUID (`doc_...`)
   - `container_id`: Workspace/project boundary isolation
   - `source_type`: `conversation`, `file`, `markdown`, `codebase`, `note`
   - `source_uri`: Reference location or origin path
   - `content_hash`: BLAKE3 256-bit hexadecimal string
   - `title`: Extracted or declared title
   - `mime_type`: Text or markdown representation
   - `created_at`, `updated_at`, `imported_at`: Unix timestamps in milliseconds

2. **Chunk**:
   - `id`: `chk_...`
   - `document_id`: Parent reference
   - `sequence`: Integer offset index
   - `content_hash`: BLAKE3 hash for chunk deduplication
   - `text`: Segment text
   - `token_count`: Estimated or tokenizer-derived token count

3. **Memory (Atomic)**:
   - `id`: `mem_...`
   - `container_id`: Partition scope
   - `canonical_text`: Clean, factual, atomic statement
   - `memory_type`: `fact`, `preference`, `episode`, `decision`, `goal`, `task`, `relationship`, `inference`, `temporary`
   - `confidence`: Floating-point `0.0 .. 1.0`
   - `importance`: Floating-point `0.0 .. 1.0`
   - `source_id`, `source_chunk_id`: Lineage and provenance pointer
   - `valid_from`, `valid_until`, `observed_at`: Temporal markers
   - `is_latest`: Boolean tracking fact supersession
   - `is_pinned`, `is_static`: Persistence overrides
   - `state`: `hot`, `warm`, `cold`, `archived`, `deleted`
   - `access_count`, `last_accessed_at`: Decay and promotion tracking

4. **Temporal Graph Edge**:
   - `source_id`, `target_id`: Memory or Entity IDs
   - `relation`: `UPDATES`, `EXTENDS`, `DERIVES`, `RELATED`, `CONTRADICTS`, `SUPPORTS`, `MENTIONS`, `BELONGS_TO`
   - `weight`: `f32`
   - `created_at`: Timestamp

5. **Entity**:
   - `id`: `ent_...`
   - `canonical_name`: Unique normalized name within container
   - `entity_type`: `person`, `project`, `organization`, `technology`, `place`, `concept`, `document`, `event`, `product`
   - `aliases`: Array of synonyms/references
   - `description`: Aggregated summary

---

## 4. Search & Retrieval Pipeline

```
Query -> Router -> Scope Prefilter -> Multi-Index Execution -> Hybrid Merge & Rerank -> Compact Context
```

1. **Router**:
   - Classifies query into `Profile`, `Exact`, `Lexical`, `Semantic`, `Temporal`, or `Hybrid`.
2. **Execution**:
   - **Profile**: Direct Key-Value retrieval (< 5ms).
   - **Lexical**: Inverted index BM25 term frequency lookup (< 20ms).
   - **Semantic**: Fast cosine similarity / quantized dot product search on embedding vectors (< 30ms).
   - **Temporal & Graph**: Traversal on active (`is_latest = true`, `valid_until = null`) nodes, expanding 1-hop connected neighbors.
3. **Scoring**:
   $$Score = w_1 \cdot Lexical + w_2 \cdot Semantic + w_3 \cdot Recency + w_4 \cdot Importance + w_5 \cdot Confidence$$
4. **Context Compiler**:
   - Deduplicates redundant memories.
   - Enforces configurable token budget (default $\le$ 2000 tokens).
   - Formats clean markdown context for AI Agents.

---

## 5. Obsidian Vault & Wiki Compiler

### 5.1 Directory Layout
```
HyperMemoryVault/
├── Home.md
├── People/
├── Projects/
├── Technologies/
├── Decisions/
├── Topics/
├── Daily/
│   └── YYYY-MM-DD.md
└── Sources/
```

### 5.2 Managed Section Markers
Every generated page strictly demarcates AI-generated data from human notes:
```markdown
---
hyper_memory_id: ent_01h...
type: technology
updated: 2026-09-19
managed: true
---
# Rust

<!-- HYPER_MEMORY:AUTO:BEGIN -->
## Facts & Status
- The core engine is implemented in Rust. [[Decisions/Rust Engine]]
- Employs INT8 quantization and memory-mapped files.

## Related
- [[Projects/Hyper Memory]]
<!-- HYPER_MEMORY:AUTO:END -->

## My Notes
(User written thoughts and manual annotations preserved across all sync cycles)
```

### 5.3 Bi-directional Synchronization & Diff Detection
- File watcher monitors `.md` updates.
- If user modifies content inside `<!-- HYPER_MEMORY:AUTO -->` or adds frontmatter tags (`#hypermemory/pin`, `#hypermemory/archive`), the engine detects semantic change candidates.
- Automated or staged review pipeline applies updates back to the Temporal Memory Graph.

---

## 6. MCP Server Protocol

Exposes standard JSON-RPC 2.0 over standard I/O (stdio) implementing:
- `memory_add`: Ingest conversational or document text.
- `memory_search`: Hybrid retrieval with semantic and temporal filters.
- `memory_get`: Inspect atomic memory details.
- `memory_forget`: Soft archive or delete memory with confidence adjustment.
- `memory_update`: Explicitly supersede or amend a memory item.
- `profile_get`: Fast retrieval of user profile, preferences, active projects.
- `wiki_search`: Markdown page search.
- `wiki_get_page`: Retrieve vault document content.
- `wiki_update_page`: Update page or append user notes.
- `project_context`: Generate compact agent prompt context under budget.
- `source_get`: Inspect provenance source chunk and document.

---

## 7. Performance & Reliability Guarantees

- **Memory Safety & Zero GC Overhead**: Implemented in idiomatic Rust.
- **Embedded Storage**: Runs self-contained with no mandatory external daemon dependencies (SQLite / Redb / In-memory indexes).
- **Fast Startup**: Sub-second cold startup.
- **Fault-Tolerant Ingestion**: Content-addressable BLAKE3 hashes ensure complete idempotency during crash recovery.
