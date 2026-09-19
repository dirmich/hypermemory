# PRD — Hyper Memory

**Document:** `prd.md`  
**Product Name:** Hyper Memory  
**Status:** v1.0 Released  
**Product Type:** Local-first / Self-hostable AI Memory & Knowledge Engine  
**Primary Interfaces:** REST API, MCP (Model Context Protocol), Obsidian Vault, CLI  
**Core Technologies:** Rust, SQLite (Bundled), Vector Search (INT8 Quantized), BM25 FTS, Temporal Graph, LLM, Markdown/Obsidian  
**Supported Platforms:** Windows, macOS, Linux (Cross-Platform)

---

## 1. Product Overview

### 1.1 One-Line Summary
Hyper Memory is a local-first AI memory and knowledge engine that extracts atomic long-term memories from conversations, code, and documents, maintains a temporal memory graph with automatic fact supersession, and deterministically projects that knowledge into human-readable, human-editable Obsidian markdown notes with bidirectional synchronization.

### 1.2 Problem Definition
Existing AI agent architectures and RAG frameworks suffer from severe structural limitations:
1. **Session Amnesia**: Conversations lose context between sessions.
2. **Context Window Inflation**: Stuffing entire past transcripts into prompt context creates exponential token cost and degradation in LLM reasoning.
3. **State Change Failure**: Naive RAG treats knowledge statically; it cannot distinguish between what was true historically and what is true *right now*.
4. **Opaque Vector DBs**: Vector databases provide similarity search but lack temporal relations, contradictions, fact supersession, and human-inspectable structure.
5. **No Human Control**: Users cannot easily verify, pin, correct, or erase facts stored by AI memory engines.
6. **Obsidian Gap**: While Obsidian is great for humans, it lacks automatic memory extraction, semantic ANN, and temporal fact tracking.
7. **Resource Bloat**: Long-term unbounded memory accumulation consumes excessive RAM.
8. **Cloud Privacy Risks**: SaaS-based memory stores leak sensitive personal or proprietary engineering context.

### 1.3 Solution Architecture
Hyper Memory combines ingestion, graph maintenance, multi-engine retrieval, and Markdown projection:

```text
Raw Inputs (Chat, Files, Notes, Code)
   ↓
Ingestion Pipeline (Normalize, Fingerprint, Dedup)
   ↓
Memory Extractor (Heuristic Noise Filter + Local LLM)
   ↓
Temporal Memory Graph (Supersession, Relations, Lifecycle)
   ↓
Entity & Topic Layer
   ↓
Wiki Compiler (Deterministic IR)
   ↓
Obsidian Vault Markdown (<!-- AUTO --> + ## My Notes)
```

At query time:
```text
Query
  ↓
Query Router (Profile, Exact, Lexical, Semantic, Temporal, Hybrid)
  ↓
Multi-Engine Search (Inverted BM25 + INT8 Vector ANN + Recency + Confidence)
  ↓
Hybrid Reranking & Graph Expansion
  ↓
Context Compiler (< 2,000 token budget)
  ↓
Compact Context for AI Agent Prompt
```

---

## 2. Core Principles

1. **Separation of Memory & Document**: Raw documents and atomic facts are stored separately.
2. **Obsidian is a Human-Facing Projection**: Ground truth resides in the Temporal Graph; Obsidian markdown is a deterministic human-facing projection.
3. **Zero Token Waste**: Conversational noise ("ok", "thanks", "ㅋㅋ") is filtered out via heuristic rules before calling LLMs.
4. **Exact/Lexical First**: If a query is an exact term or profile lookup, do not waste compute on vector ANN.
5. **Memory Tiering**: Unused memories decay and migrate through Hot, Warm, Cold, and Archived tiers.
6. **Temporal Awareness as a Core Primitive**: Every memory tracks `valid_from`, `valid_until`, and `is_latest`.
7. **Zero Loss of User Notes**: AI modifies strictly inside `<!-- HYPER_MEMORY:AUTO -->`, preserving 100% of human notes.
8. **Local-First & Cross-Platform**: Operates fully offline without internet; compiles natively for Windows, macOS, and Linux.
9. **Low Memory Footprint**: Uses bundled SQLite and INT8 quantized vectors (< 15MB base RAM).
10. **Universal Agent Interoperability**: Compatible with Claude Code, Cursor, Codex, and any tool supporting the Anthropic Model Context Protocol (MCP) or REST.

---

## 3. Storage & Domain Models

### 3.1 Document
- `id`: Content-addressable identifier (`doc_...`)
- `container_id`: Workspace/project boundary
- `source_type`: `conversation`, `file`, `markdown`, `codebase`, `note`
- `source_uri`: Reference URI or file path
- `content_hash`: BLAKE3 256-bit hexadecimal hash
- `title`, `mime_type`, `metadata`
- `created_at`, `updated_at`, `imported_at`

### 3.2 Chunk
- `id`: `chk_...`
- `document_id`: Parent reference
- `sequence`: Integer offset
- `content_hash`: BLAKE3 chunk hash for deduplication
- `text`: Chunk content
- `token_count`: Estimated token count

### 3.3 Memory
- `id`: `mem_...`
- `container_id`: Namespace scope
- `canonical_text`: Clean atomic statement
- `memory_type`: `fact`, `preference`, `episode`, `decision`, `goal`, `task`, `relationship`, `inference`, `temporary`
- `confidence`: `0.0 .. 1.0`
- `importance`: `0.0 .. 1.0`
- `source_id`, `source_chunk_id`: Provenance pointers
- `valid_from`, `valid_until`: Temporal validity markers
- `is_latest`: Boolean tracking fact supersession
- `is_pinned`, `is_static`: User overrides
- `state`: `hot`, `warm`, `cold`, `archived`, `deleted`

### 3.4 Edge
- `source_id`, `target_id`: Memory or Entity IDs
- `relation`: `UPDATES`, `EXTENDS`, `DERIVES`, `RELATED`, `CONTRADICTS`, `SUPPORTS`, `MENTIONS`, `BELONGS_TO`
- `weight`: `f32`

### 3.5 Entity
- `id`: `ent_...`
- `canonical_name`: Unique name within container
- `entity_type`: `person`, `project`, `organization`, `technology`, `place`, `concept`, `document`, `event`, `product`
- `aliases`: Synonyms and wikilinks

---

## 4. Temporal Memory Graph & Fact Supersession

### 4.1 Automatic Fact Updating
When a new memory supersedes an existing one (e.g. switching databases or changing preferences):
1. Previous memory has `is_latest` set to `false` and `valid_until` stamped.
2. New memory has `is_latest = true` and `valid_from` stamped.
3. Edge created: `NewMemory --UPDATES--> OldMemory`.

### 4.2 Memory Consolidation
Repeated or similar statements regarding an entity are clustered and synthesized into a single canonical fact, while archiving the original memories (`Canonical --DERIVES--> Originals`).

### 4.3 Memory Lifecycle & Tiers
- **Hot**: Recent (< 30 days), high access count, or `is_pinned = true`.
- **Warm**: 30 days to 1 year, moderate access.
- **Cold**: > 1 year, low access.
- **Decay**: Temporary and episodic memories decay importance over time unless accessed or pinned.

---

## 5. Multi-Engine Search & Context Compiler

### 5.1 Query Routing
Classifies query into:
- `Profile`: KV lookup (`< 5ms`)
- `Exact`: Exact phrase lookup
- `Lexical`: Inverted index BM25 lookup (`< 20ms`)
- `Temporal`: Traversal including historical superseded memories
- `Hybrid`: Weighted multi-engine score

### 5.2 Hybrid Ranking Formula
$$Score = 0.35 \cdot BM25_{norm} + 0.35 \cdot Vector_{cos} + 0.15 \cdot Recency + 0.10 \cdot Importance + 0.05 \cdot Confidence$$

### 5.3 Context Compiler
Limits prompt context strictly under **2,000 tokens**, packing active facts, decisions, and profile notes.

---

## 6. Obsidian Living Wiki & Two-Way Sync

### 6.1 Vault Folder Structure
- `People/`, `Projects/`, `Technologies/`, `Decisions/`, `Topics/`, `Daily/YYYY-MM-DD.md`, `Sources/`.

### 6.2 Section Isolation
```markdown
---
hyper_memory_id: ent_931e87a4a1e4c737
type: Technology
managed: true
---
# Technology Name

<!-- HYPER_MEMORY:AUTO:BEGIN -->
## Summary
...
## Facts & Status
- Fact 1
## Related
- [[Project]]
<!-- HYPER_MEMORY:AUTO:END -->

## My Notes
(User writes notes here. 100% preserved across sync cycles!)
```

### 6.3 Control Tags
- `#hypermemory/pin` — Retain in Hot tier.
- `#hypermemory/archive` — Move memory to archive.
- `#hypermemory/no-ai` — Exclude from AI indexing.

---

## 7. Interfaces & Protocols

### 7.1 REST API
- `POST /v1/documents` — Ingest document
- `POST /v1/search` — Hybrid search
- `POST /v1/context` — Compact agent context
- `GET  /v1/memories` — List memories
- `GET  /v1/profile/:container` — Profile lookup
- `POST /v1/wiki/sync` — Synchronize Obsidian Vault
- `GET  /health` — Health check

### 7.2 MCP (Model Context Protocol)
Implements standard JSON-RPC 2.0 stdio protocol with 11 tools:
`memory_add`, `memory_search`, `memory_get`, `memory_forget`, `memory_update`, `profile_get`, `wiki_search`, `wiki_get_page`, `wiki_update_page`, `project_context`, `source_get`.

### 7.3 CLI Binary
Subcommands: `start`, `mcp`, `add`, `search`, `context`, `wiki-sync`, `doctor`, `benchmark`.

---

## 8. Success Criteria & Benchmarks

1. **Ingestion Speed**: < 2ms per fact.
2. **Search Latency**: p95 < 5ms without LLM rerank.
3. **Deduplication Rate**: > 95% duplicate rejection.
4. **Zero Note Loss**: 0 user notes lost in Obsidian round-trip synchronization.
5. **Memory Footprint**: < 20MB base RAM with INT8 quantization.
6. **Cross-Platform**: Zero external C runtime dependencies; bundled SQLite on Windows, macOS, and Linux.
