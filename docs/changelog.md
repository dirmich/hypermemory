# Changelog

All notable changes to the Hyper Memory project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
