# Changelog

All notable changes to the Hyper Memory project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - 2026-09-19
### Added
- Complete Korean usage guide (`docs/usage-kor.md`) explaining CLI commands, REST API integration, MCP configuration, and Obsidian Vault bidirectional synchronization.
- Linked Korean usage guide in `README.md`.

## [0.1.8] - 2026-09-19
### Added
- Comprehensive English usage guide (`docs/usage.md`) covering CLI commands, REST API endpoints with JSON examples, MCP setup, and Obsidian Vault integration.
- Full English Product Requirements Document (`docs/prd.md`).
- Renamed original Korean PRD to `docs/prd-kor.md`.

## [0.1.7] - 2026-09-19
### Added
- Unified CLI binary (`hyper-memory`) with subcommands: `start`, `mcp`, `add`, `search`, `context`, `wiki-sync`, `doctor`, `benchmark`.
- Automated performance and recall benchmark suite measuring ingestion speed (~1.3ms), dedup rate (100%), hybrid search latency (~1.3ms), and temporal fact supersession.
- Comprehensive `README.md` documenting core strengths: Persistent AI Memory + Living Wiki, Temporal Graph with Fact Supersession, Zero Token Waste, Cross-platform Local-First engine, and Universal MCP / REST interoperability.

## [0.1.6] - 2026-09-19
### Added
- Axum REST API server with endpoints: `/health`, `/v1/documents`, `/v1/search`, `/v1/context`, `/v1/memories`, `/v1/profile/:container`, `/v1/wiki/sync`.
- Cross-origin CORS support and JSON payload request/response models.
- Standard Anthropic Model Context Protocol (MCP) stdio JSON-RPC 2.0 server (`McpServer`).
- Complete suite of 11 MCP tools: `memory_add`, `memory_search`, `memory_get`, `memory_forget`, `memory_update`, `profile_get`, `wiki_search`, `wiki_get_page`, `wiki_update_page`, `project_context`, `source_get`.

## [0.1.5] - 2026-09-19
### Added
- Structured Wiki Intermediate Representation (IR) compiler (`WikiCompiler`, `WikiPageIR`).
- Strict section demarcation preserving human notes outside `<!-- HYPER_MEMORY:AUTO -->` markers.
- Cross-platform safe atomic vault writer preventing file corruption (`VaultWriter`).
- Obsidian bidirectional sync engine supporting forward export, reverse import, and `#hypermemory/*` control tags (`ObsidianSyncEngine`).

## [0.1.4] - 2026-09-19
### Added
- Embedded BM25 Inverted Index Full-Text Search (`BM25InvertedIndex`).
- Vector storage with cosine similarity and FP32/INT8 quantization for low RAM footprint (`VectorIndex`, `StoredVector`).
- Query classification and routing (`QueryRouter`).
- Hybrid search scoring combining lexical, vector, recency, importance, and confidence (`HybridSearchEngine`).
- Strict token budget context compiler generating compact markdown prompts for AI agents (`ContextCompiler`).

## [0.1.3] - 2026-09-19
### Added
- Temporal update engine with automatic fact supersession (`is_latest = false`), contradiction detection, and relation edges (`TemporalUpdateEngine`).
- Memory lifecycle manager handling Hot/Warm/Cold tier migrations, pinning, recency decay, and access promotion (`MemoryLifecycleManager`).
- Memory consolidation engine merging repeated/similar memories into canonical facts while archiving originals (`MemoryConsolidationEngine`).

## [0.1.2] - 2026-09-19
### Added
- Text normalization with Unicode NFKC and whitespace collapsing (`ContentNormalizer`).
- Sliding-window and markdown-aware chunking (`ContentChunker`).
- Content-fingerprint deduplication and ingestion pipeline (`IngestionPipeline`).
- Cheap noise filter for filtering out conversational chatter before LLM calls (`CheapNoiseFilter`).
- Deterministic local LLM and 384d embedding providers (`LocalRuleBasedLLM`, `LocalDeterministicEmbedding`).
- End-to-end memory extraction engine linking memories to entities and provenance sources (`MemoryExtractionEngine`).

## [0.1.1] - 2026-09-19
### Added
- Domain models: `Document`, `Chunk`, `Memory`, `Entity`, `Edge`, `UserProfile`.
- Bundled cross-platform SQLite storage engine (`SqliteStorage`) with WAL mode, foreign keys, and indexes.
- StorageBackend traits covering documents, chunks, memories, entities, edges, and profiles.
- BLAKE3 content-addressable ID and hash generation.
- Automated unit tests for document, chunk, memory, and edge CRUD operations.

## [0.1.0] - 2026-09-19
### Added
- Initial planning documentation: `docs/plan.md`, `docs/task.md`.
- Architecture blueprints for Temporal Memory Graph, Vector Search, Obsidian Two-way Sync, and MCP Server.
