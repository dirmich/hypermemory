# Hyper Memory

<div align="center">

**Local-First Temporal AI Memory & Human-Editable Living Knowledge Engine**

[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)](https://github.com/dirmich/hypermemory)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/language-Rust%202021-orange.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Anthropic%20Compatible-purple.svg)](https://modelcontextprotocol.io)

</div>

---

## 💡 What is Hyper Memory?

Traditional AI agents suffer from acute memory amnesia: sessions expire, past context gets forgotten, or token costs explode when dumping entire transcripts into prompts. Vector databases offer similarity search but fundamentally lack a sense of **time**, **state change**, **contradiction**, and **human inspectability**.

**Hyper Memory** bridges this divide. It automatically extracts atomic facts, decisions, and entities from conversations, builds an incrementally updated **Temporal Knowledge Graph**, and deterministically projects that graph into a human-readable, human-editable **Obsidian Vault / Markdown Wiki**. When you edit your Obsidian notes, Hyper Memory seamlessly syncs changes back to the AI's ground-truth memory.

```
       +------------------------------------------------------------------+
       |                           AI Agents                              |
       |             Claude Code · Cursor · Codex · Hermes · CLI          |
       +--------------------------------+---------------------------------+
                                        |
                         +--------------+--------------+
                         | REST API (Axum) / MCP Server|
                         +--------------+--------------+
                                        |
       +--------------------------------v---------------------------------+
       |                      Hyper Memory Core Engine                    |
       |                                                                  |
       |   Ingestion & Dedup  --> Memory Extractor --> Temporal Graph     |
       |   (BLAKE3 / Normal)      (Heuristic Filter)   (Fact Updates)     |
       |                                                                  |
       |   Hybrid Retrieval   <-- Vector ANN (INT8) <-- Inverted BM25     |
       |   (Context Compiler)     (Zero RAM bloat)      (Sub-millisecond) |
       +--------------------------------+---------------------------------+
                                        |
                                        v
                       +---------------------------------+
                       |     Obsidian Living Wiki        |
                       |       <!-- AUTO -->             |
                       |   Human-Editable Markdown       |
                       +---------------------------------+
```

---

## ⚡ Greatest Strengths & Key Advantages

### 1. 🧠 Persistent Machine Memory $\leftrightarrow$ Human-Editable Living Wiki
- **Not a Black Box**: Memory is not locked inside an opaque vector cloud. Every memory is compiled into clear, structured Markdown pages (`Technologies/`, `Projects/`, `Decisions/`, `Daily/`).
- **Strict Section Demarcation**: AI automatically maintains its designated section (`<!-- HYPER_MEMORY:AUTO:BEGIN -->` ... `<!-- HYPER_MEMORY:AUTO:END -->`), while preserving **100% of human notes and thoughts** under `## My Notes`.
- **Bidirectional Synchronization**: Edit your notes in Obsidian, use control tags (`#hypermemory/pin`, `#hypermemory/archive`, `#hypermemory/no-ai`), and Hyper Memory's semantic diff engine updates the underlying graph.

### 2. ⏳ Temporal Graph with Automatic Fact Supersession
- **Tracks What is True Right Now**: Knows that *"We chose PostgreSQL"* $\rightarrow$ *"We switched to ClickHouse for analytics"* means PostgreSQL is no longer current (`is_latest = false`).
- **Relation Graph**: Connects nodes via `UPDATES`, `EXTENDS`, `DERIVES`, `CONTRADICTS`, `SUPPORTS`, and `MENTIONS`.
- **Contradiction Resolution & Memory Consolidation**: Resolves conflicting statements and consolidates repeated facts into canonical memories with full provenance tracing back to original sources.

### 3. 🎯 Zero Token Waste & Strict Token Budgeting
- **Cheap Heuristic Noise Filter**: Pre-filters conversational chatter (*"yes"*, *"thanks"*, *"ok"*, *"got it"*) without expensive LLM roundtrips.
- **Compact Context Compiler**: Automatically limits prompt injections under **2,000 tokens**, packing only active facts, relevant user profiles, and provenance pointers.

### 4. 💻 True Local-First & Cross-Platform (Windows, macOS, Linux)
- **Zero Cloud Lock-in**: Fully functional offline. Ships with local deterministic embedding and rule-based extraction engines, with support for Ollama, vLLM, and OpenAI-compatible endpoints.
- **Low Memory Footprint**: Bundled embedded SQLite (WAL mode) with **INT8 vector quantization** allowing instant cosine similarity matching with 4x less memory consumption.
- **Cross-Platform Native**: Tested and built cleanly for Windows (x86_64, aarch64), macOS (Apple Silicon, Intel), and Linux.

### 5. 🔌 Universal Interoperability (MCP & REST API)
- **Model Context Protocol (MCP)**: Implements 11 first-class MCP tools: `memory_add`, `memory_search`, `memory_get`, `memory_forget`, `memory_update`, `profile_get`, `wiki_search`, `wiki_get_page`, `wiki_update_page`, `project_context`, and `source_get`.
- **RESTful HTTP API**: High-throughput Axum server with full CORS support and typed JSON schemas.

---

## 📦 Downloads & Pre-built Binaries

Pre-compiled standalone binaries are available on the [GitHub Releases](https://github.com/dirmich/hypermemory/releases) page for instant execution without needing to install Rust:

| Platform | Architecture | Download Link | Format |
|---|---|---|---|
| **macOS** | Apple Silicon (`aarch64`) | [Download macOS Binary](https://github.com/dirmich/hypermemory/releases/download/v0.1.7/hyper-memory-v0.1.7-macos-aarch64.tar.gz) | `.tar.gz` |
| **Linux** | x86_64 (`gnu`) | [Download Linux Binary](https://github.com/dirmich/hypermemory/releases) | `.tar.gz` |
| **Windows** | x86_64 (`msvc`) | [Download Windows Binary](https://github.com/dirmich/hypermemory/releases) | `.zip` |

For detailed documentation, refer to the [Complete Usage Guide](docs/usage.md) ([한국어 사용설명서](docs/usage-kor.md)) and [Product Requirements Document](docs/prd.md) ([한국어 PRD](docs/prd-kor.md)).

---

## 🚀 Quick Start & Installation

### Option A: Download Pre-built Binary
Extract and run directly:
```bash
# macOS / Linux
tar -xvf hyper-memory-*.tar.gz
chmod +x hyper-memory
./hyper-memory doctor

# Windows (PowerShell)
Expand-Archive -Path hyper-memory-*.zip -DestinationPath .
.\hyper-memory.exe doctor
```

### Option B: Build from Source
Ensure Rust 1.80+ is installed, then clone and build:

```bash
git clone https://github.com/dirmich/hypermemory.git
cd hypermemory
cargo build --release
```

The executable will be located at `target/release/hyper-memory`.

---

## 🛠️ Essential Usage Guide

> [!TIP]
> For the complete documentation with all REST API endpoints, JSON payloads, and Claude Desktop MCP configurations, read the **[docs/usage.md](docs/usage.md)**.

### 1. Ingest Conversation or Notes
```bash
hyper-memory add --container "my-project" "We decided to build the core engine in Rust."
```

### 2. Hybrid Search
```bash
hyper-memory search --container "my-project" "Rust engine"
```

### 3. Generate Compact Agent Prompt Context (< 2000 tokens)
```bash
hyper-memory context --container "my-project" "What technologies are we using?"
```

### 4. Sync with Obsidian Vault (Two-Way)
```bash
hyper-memory wiki-sync --container "my-project" --vault-path ~/Documents/ObsidianVault
```

### 5. Start Background REST API Server
```bash
hyper-memory start --port 6767
```

### 6. Run as Anthropic Model Context Protocol (MCP) Stdio Server
```bash
hyper-memory mcp
```

### 7. Run System Health & Diagnosis
```bash
hyper-memory doctor
```

### 8. Run Performance & Correctness Benchmark
```bash
hyper-memory benchmark
```

---

## 🌐 REST API Server

Start the REST API server on default port `6767`:

```bash
hyper-memory start --port 6767
```

### Core Endpoints:
- `POST /v1/documents` — Ingest document or raw transcript
- `POST /v1/search` — Hybrid search (Lexical BM25 + Vector ANN + Recency + Confidence)
- `POST /v1/context` — Compile compact agent context under token budget
- `GET  /v1/memories?container=...` — Retrieve active and historical memories
- `GET  /v1/profile/:container` — Key-value profile lookup
- `POST /v1/wiki/sync` — Synchronize Obsidian Vault
- `GET  /health` — Service healthcheck

---

## 🤖 MCP (Model Context Protocol) Setup

Hyper Memory runs as a standard MCP stdio server. Add it to your MCP client configuration (e.g. Claude Desktop, Claude Code, Cursor, OpenCode):

```json
{
  "mcpServers": {
    "hypermemory": {
      "command": "/path/to/target/release/hyper-memory",
      "args": ["mcp", "--data-dir", "/path/to/.hyper-memory", "--vault-path", "/path/to/HyperMemoryVault"]
    }
  }
}
```

### Available Tools:
| Tool Name | Description |
|---|---|
| `memory_add` | Ingest conversational text and extract facts/decisions |
| `memory_search` | Multi-engine hybrid search with temporal awareness |
| `memory_get` | Retrieve full atomic memory object by ID |
| `memory_update` | Update canonical text or toggle pin status |
| `memory_forget` | Soft-archive or forget memory item |
| `profile_get` | Fast KV lookup of user/project profile |
| `project_context`| Generate compact prompt context within token budget |
| `wiki_search` | Search markdown pages across the Obsidian vault |
| `wiki_get_page` | Read markdown content of vault page |
| `wiki_update_page`| Trigger bidirectional Obsidian vault synchronization |
| `source_get` | Inspect provenance document and chunk references |

---

## 📊 Performance Benchmarks

Results on an Apple Silicon M-series machine (built-in benchmark):
- **Ingestion Throughput**: ~1.3ms per fact (BLAKE3 hash + normalize + extract + temporal update)
- **Content Deduplication**: 100% duplicate rejection in ~1.1ms
- **Hybrid Search Latency**: ~1.3ms p95 query latency (BM25 + quantized vector + scoring)
- **Memory Footprint**: < 15MB base RAM with INT8 quantization
- **Vault Note Loss**: 0 user notes lost across re-compilation cycles

---

## 📂 Obsidian Vault Architecture

```
HyperMemoryVault/
├── Home.md
├── People/
├── Projects/
├── Technologies/
│   └── Rust.md
├── Decisions/
│   └── Rust Core Engine.md
├── Topics/
├── Daily/
│   └── 2026-09-19.md
└── Sources/
```

Inside any generated `.md` note:
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
(Write personal notes, annotations, or append #hypermemory/pin here. Your notes are 100% preserved during sync!)
```

---

## 📄 License

Dual-licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
