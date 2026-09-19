# Hyper Memory — Usage Guide

This document provides a comprehensive operational guide for Hyper Memory, covering CLI commands, REST API integration, Model Context Protocol (MCP) setup for AI agents, and Obsidian Vault bidirectional synchronization.

---

## 1. Quick Installation & Build

### Pre-requisites
- **Rust toolchain** (1.80+): Install via [rustup.rs](https://rustup.rs/)
- Supported operating systems: **Windows**, **macOS**, **Linux**

### Build from Source
```bash
git clone https://github.com/dirmich/hypermemory.git
cd hypermemory
cargo build --release
```

The compiled binary will be placed at:
- **macOS / Linux**: `target/release/hyper-memory`
- **Windows**: `target/release/hyper-memory.exe`

You can optionally install it into your system PATH:
```bash
cargo install --path .
```

---

## 2. CLI Command Reference

Hyper Memory comes with a unified command-line tool `hyper-memory`.

### 2.1 Help & Subcommands
```bash
hyper-memory --help
```

Global Options:
- `-d, --data-dir <PATH>`: Directory for SQLite database and indexes (default: `./.hyper-memory`)
- `-v, --vault-path <PATH>`: Directory for the Obsidian vault (default: `./HyperMemoryVault`)

---

### 2.2 Adding & Ingesting Information (`add`)
Ingest conversational text, statements, or decisions into a container (workspace/project namespace):

```bash
# Ingest into default container
hyper-memory add "We decided to build Hyper Memory in Rust for predictable memory safety."

# Ingest into a specific project container
hyper-memory add --container "my-project" "Switched the analytical database from PostgreSQL to ClickHouse."
```

Output:
```text
✅ Ingested document: doc_8a8d091474f6aaf7
   Extracted 1 memories into container 'my-project'.
   - [Decision] Switched the analytical database from PostgreSQL to ClickHouse
```

---

### 2.3 Hybrid Search (`search`)
Search memories across full-text BM25, INT8 quantized vector similarity, recency, and temporal awareness:

```bash
hyper-memory search --container "my-project" "database"
```

Output:
```text
🔍 Search results for "database" (Container: my-project):
   1. [0.512] [Decision] Switched the analytical database from PostgreSQL to ClickHouse
```

Options:
- `-c, --container <NAME>`: Namespace scope (default: `default`)
- `-l, --limit <NUMBER>`: Maximum results to return (default: `5`)

---

### 2.4 Context Compiler for AI Agents (`context`)
Generate a token-budgeted, concise Markdown prompt context ready to be fed directly into an LLM or agent system prompt:

```bash
hyper-memory context --container "my-project" "What database are we using?" --max-tokens 1500
```

Output:
```markdown
# Hyper Memory Context (Budget: 1500 tokens, Used: ~25 tokens)

## Relevant Current Memories
- [Decision] Switched the analytical database from PostgreSQL to ClickHouse (conf: 0.95, imp: 0.90)

## Sources & Provenance
- Memory `mem_1603d84ad1dc3a07` -> Source `doc_8a8d091474f6aaf7`
```

---

### 2.5 Obsidian Vault Synchronization (`wiki-sync`)
Performs bidirectional synchronization between the SQLite storage engine and your Obsidian Vault:

```bash
hyper-memory wiki-sync --container "my-project" --vault-path ~/Documents/ObsidianVault
```

Output:
```text
🔄 Obsidian Vault Synchronization Complete:
   - Vault path: "/Users/.../Documents/ObsidianVault"
   - Exported pages: 4
   - Imported memories from vault notes: 2
   - Control tags processed: 1
```

---

### 2.6 System Doctor & Diagnostics (`doctor`)
Inspects system health, directory permissions, database connectivity, and document counts:

```bash
hyper-memory doctor
```

---

### 2.7 Performance & Recall Benchmark (`benchmark`)
Runs the built-in end-to-end benchmark verifying ingestion throughput, duplicate rejection, hybrid search latency, and temporal fact supersession:

```bash
hyper-memory benchmark
```

---

### 2.8 Starting the REST API Server (`start`)
Launches the high-throughput Axum HTTP server:

```bash
hyper-memory start --port 6767
```

---

### 2.9 Starting the MCP Server (`mcp`)
Launches the Anthropic Model Context Protocol server over standard I/O (stdio):

```bash
hyper-memory mcp
```

---

## 3. REST API Documentation

Base URL: `http://localhost:6767`

### 3.1 Ingest Document
- **Endpoint**: `POST /v1/documents`
- **Request Body**:
```json
{
  "container": "project_alpha",
  "content": "We decided to adopt ClickHouse for our high-throughput analytics.",
  "title": "Database Architecture Decision",
  "source_type": "conversation"
}
```
- **Response**:
```json
{
  "document_id": "doc_2f8a1c90",
  "is_duplicate": false,
  "chunk_count": 1,
  "extracted_memories": 1
}
```

---

### 3.2 Hybrid Search
- **Endpoint**: `POST /v1/search`
- **Request Body**:
```json
{
  "container": "project_alpha",
  "query": "Which database for analytics?",
  "limit": 5
}
```
- **Response**:
```json
{
  "query": "Which database for analytics?",
  "hits": [
    {
      "id": "mem_4a8b7c",
      "text": "We decided to adopt ClickHouse for our high-throughput analytics",
      "memory_type": "decision",
      "score": 0.584,
      "match_sources": ["lexical", "semantic"]
    }
  ]
}
```

---

### 3.3 Compile Agent Context
- **Endpoint**: `POST /v1/context`
- **Request Body**:
```json
{
  "container": "project_alpha",
  "query": "Current database stack",
  "max_tokens": 1500
}
```
- **Response**:
```json
{
  "context": "# Hyper Memory Context ...",
  "query": "Current database stack"
}
```

---

### 3.4 List Active Memories
- **Endpoint**: `GET /v1/memories?container=project_alpha`

---

### 3.5 Profile Lookup
- **Endpoint**: `GET /v1/profile/project_alpha`

---

### 3.6 Synchronize Obsidian Vault
- **Endpoint**: `POST /v1/wiki/sync`
- **Request Body**:
```json
{
  "container": "project_alpha"
}
```

---

### 3.7 Healthcheck
- **Endpoint**: `GET /health`
- **Response**: `200 OK`

---

## 4. MCP (Model Context Protocol) Integration

Hyper Memory natively implements the Anthropic MCP protocol over standard I/O (stdio). This allows tools like **Claude Code**, **Claude Desktop**, **Cursor**, and **OpenCode** to access long-term persistent memory seamlessly.

### 4.1 Claude Desktop Configuration
Add the server definition to your `claude_desktop_config.json`:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "hypermemory": {
      "command": "/path/to/hyper-memory",
      "args": [
        "mcp",
        "--data-dir", "/path/to/.hyper-memory",
        "--vault-path", "/path/to/HyperMemoryVault"
      ]
    }
  }
}
```

### 4.2 Claude Code Configuration
Run in your project directory:
```bash
claude mcp add hypermemory -- /path/to/hyper-memory mcp
```

### 4.3 Available MCP Tools
| Tool Name | Key Parameters | Description |
|---|---|---|
| `memory_add` | `container`, `text` | Ingest conversational text or facts into memory |
| `memory_search` | `container`, `query`, `limit` | Hybrid search across memories with temporal awareness |
| `memory_get` | `id` | Retrieve full atomic memory object |
| `memory_update` | `id`, `canonical_text`, `is_pinned` | Update memory text or toggle pin |
| `memory_forget` | `id` | Soft-archive memory |
| `profile_get` | `container` | Get user/project profile and preferences |
| `wiki_search` | `container`, `query` | Search markdown pages in vault |
| `wiki_get_page` | `folder`, `title` | Read markdown content of vault page |
| `wiki_update_page`| `container` | Trigger two-way Obsidian sync |
| `project_context`| `container`, `query`, `max_tokens` | Generate compact prompt context within token budget |
| `source_get` | `document_id` | Inspect original document and chunk sources |

---

## 5. Obsidian Vault Integration & Two-Way Sync

Hyper Memory projects ground-truth graph knowledge into a clean, human-readable directory of Markdown files.

### 5.1 Vault Directory Structure
```
HyperMemoryVault/
├── Home.md
├── People/
│   └── Alice.md
├── Projects/
│   └── Hyper Memory.md
├── Technologies/
│   ├── Rust.md
│   └── ClickHouse.md
├── Decisions/
│   └── Database Migration.md
├── Topics/
├── Daily/
│   └── 2026-09-19.md
└── Sources/
```

### 5.2 Safe Section Isolation
Every generated file divides AI-managed content from user notes:

```markdown
---
hyper_memory_id: ent_931e87a4a1e4c737
type: Technology
updated: 2026-09-19
managed: true
---

# Rust

<!-- HYPER_MEMORY:AUTO:BEGIN -->
## Summary
Documentation and facts for Rust.

## Decisions
- We decided to build Hyper Memory in Rust

## Sources
- [[Sources/doc_8a8d091474f6aaf7]]
<!-- HYPER_MEMORY:AUTO:END -->

## My Notes
Write your own notes, comments, and reflections here!
Your notes are 100% preserved whenever Hyper Memory regenerates the AUTO block.
```

### 5.3 Control Tags
You can insert special control tags anywhere in your Obsidian notes:
- `#hypermemory/pin` — Retains the memory permanently in the **Hot** tier, preventing decay or archiving.
- `#hypermemory/archive` — Soft-archives the memory into cold storage.
- `#hypermemory/no-ai` — Excludes the note from AI ingestion and indexing.

---

## 6. Configuration Best Practices

For server deployments, consider organizing your data directories as follows:

```bash
# Recommended directory layout:
~/.hyper-memory/
├── hypermemory.db          # Main SQLite WAL database
├── hypermemory.db-wal      # Write-Ahead Log
└── HyperMemoryVault/       # Target Obsidian vault directory
```

Launch the service in background or as a systemd service / launchd daemon:
```bash
hyper-memory start --port 6767 --data-dir ~/.hyper-memory --vault-path ~/.hyper-memory/HyperMemoryVault
```
