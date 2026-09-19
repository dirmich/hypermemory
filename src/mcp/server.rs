use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::api::handlers::AppState;
use crate::domain::{MemoryState, SourceType};
use crate::extractor::engine::MemoryExtractionEngine;
use crate::ingestion::pipeline::IngestionPipeline;
use crate::search::context_compiler::{ContextCompiler, ContextCompilerOptions};
use crate::search::hybrid::HybridSearchEngine;
use crate::temporal::updater::TemporalUpdateEngine;
use crate::wiki::sync::ObsidianSyncEngine;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::sync::Arc;

pub struct McpServer {
    state: Arc<AppState>,
}

impl McpServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id;
        match req.method.as_str() {
            "initialize" => JsonRpcResponse::success(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "hypermemory",
                        "version": crate::VERSION
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }),
            ),
            "notifications/initialized" => JsonRpcResponse::success(id, json!({})),
            "tools/list" => JsonRpcResponse::success(id, json!({ "tools": self.list_tools() })),
            "tools/call" => {
                let params = match req.params {
                    Some(p) => p,
                    None => return JsonRpcResponse::error(id, -32602, "Missing params"),
                };
                let name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => return JsonRpcResponse::error(id, -32602, "Missing tool name"),
                };
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.execute_tool(name, args) {
                    Ok(res) => JsonRpcResponse::success(
                        id,
                        json!({
                            "content": [{
                                "type": "text",
                                "text": res
                            }]
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(id, -32000, e),
                }
            }
            _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", req.method)),
        }
    }

    fn list_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "memory_add",
                "description": "Ingest and extract facts, decisions, and atomic memories from conversation or text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Project or user namespace ID" },
                        "text": { "type": "string", "description": "Text content to ingest and remember" }
                    },
                    "required": ["container", "text"]
                }
            }),
            json!({
                "name": "memory_search",
                "description": "Hybrid search across memories using semantic, lexical, recency and temporal ranking.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Project or user namespace ID" },
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "integer", "description": "Max results to return" }
                    },
                    "required": ["container", "query"]
                }
            }),
            json!({
                "name": "memory_get",
                "description": "Retrieve details of a single memory by ID.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Memory ID" }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "memory_forget",
                "description": "Archive or soft-delete a memory item.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Memory ID to forget" }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "memory_update",
                "description": "Update canonical text or pin/unpin an existing memory.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Memory ID" },
                        "canonical_text": { "type": "string", "description": "Updated fact text" },
                        "is_pinned": { "type": "boolean", "description": "Pin status" }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "profile_get",
                "description": "Retrieve the user or project profile, active goals, and preferences.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Project or user namespace ID" }
                    },
                    "required": ["container"]
                }
            }),
            json!({
                "name": "wiki_search",
                "description": "Search Obsidian vault wiki markdown pages.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Container ID" },
                        "query": { "type": "string", "description": "Wiki title or keyword" }
                    },
                    "required": ["container", "query"]
                }
            }),
            json!({
                "name": "wiki_get_page",
                "description": "Get markdown content of an Obsidian wiki page.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "folder": { "type": "string", "description": "Subfolder (e.g. Technologies, Projects)" },
                        "title": { "type": "string", "description": "Page title" }
                    },
                    "required": ["folder", "title"]
                }
            }),
            json!({
                "name": "wiki_update_page",
                "description": "Sync or trigger Obsidian vault build.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Container ID" }
                    },
                    "required": ["container"]
                }
            }),
            json!({
                "name": "project_context",
                "description": "Generate compact agent prompt context within token budget.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Container ID" },
                        "query": { "type": "string", "description": "Current agent goal or query" },
                        "max_tokens": { "type": "integer", "description": "Max token budget" }
                    },
                    "required": ["container", "query"]
                }
            }),
            json!({
                "name": "source_get",
                "description": "Inspect provenance document and chunk source text.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "document_id": { "type": "string", "description": "Document ID" }
                    },
                    "required": ["document_id"]
                }
            }),
        ]
    }

    fn execute_tool(&self, name: &str, args: Value) -> Result<String, String> {
        let rt = tokio::runtime::Handle::current();

        match name {
            "memory_add" => {
                let container = args["container"].as_str().ok_or("Missing container")?;
                let text = args["text"].as_str().ok_or("Missing text")?;

                let state = self.state.clone();
                let c = container.to_string();
                let t = text.to_string();

                rt.block_on(async move {
                    let pipeline = IngestionPipeline::new(state.storage.clone());
                    let doc_res = pipeline
                        .ingest_text(&c, &t, SourceType::Conversation, None, None)
                        .await
                        .map_err(|e| e.to_string())?;

                    let extractor = MemoryExtractionEngine::new(state.storage.clone(), state.llm.clone());
                    let mut mems = extractor
                        .extract_and_store(&c, &t, Some(doc_res.document.id), None)
                        .await
                        .map_err(|e| e.to_string())?;

                    let updater = TemporalUpdateEngine::new(state.storage.clone());
                    for m in &mut mems {
                        let _ = updater.process_temporal_relations(m).await;
                    }

                    Ok(format!(
                        "Successfully ingested and extracted {} memories into container '{}'.",
                        mems.len(),
                        c
                    ))
                })
            }
            "memory_search" => {
                let container = args["container"].as_str().ok_or("Missing container")?;
                let query = args["query"].as_str().ok_or("Missing query")?;
                let limit = args["limit"].as_u64().unwrap_or(5) as usize;

                let state = self.state.clone();
                let c = container.to_string();
                let q = query.to_string();

                rt.block_on(async move {
                    let engine = HybridSearchEngine::new(state.storage.clone(), state.embedder.clone());
                    let hits = engine.search(&c, &q, limit).await.map_err(|e| e.to_string())?;

                    let mut out = String::new();
                    for (i, h) in hits.iter().enumerate() {
                        out.push_str(&format!(
                            "{}. [{}] {} (score: {:.3})\n",
                            i + 1,
                            h.memory.id,
                            h.memory.canonical_text,
                            h.score
                        ));
                    }
                    if out.is_empty() {
                        out = "No matching memories found.".to_string();
                    }
                    Ok(out)
                })
            }
            "memory_get" => {
                let id = args["id"].as_str().ok_or("Missing id")?;
                let state = self.state.clone();
                let i = id.to_string();

                rt.block_on(async move {
                    let mem = state.storage.get_memory(&i).await.map_err(|e| e.to_string())?;
                    match mem {
                        Some(m) => Ok(serde_json::to_string_pretty(&m).unwrap_or_default()),
                        None => Err(format!("Memory '{}' not found", i)),
                    }
                })
            }
            "memory_forget" => {
                let id = args["id"].as_str().ok_or("Missing id")?;
                let state = self.state.clone();
                let i = id.to_string();

                rt.block_on(async move {
                    if let Some(mut mem) = state.storage.get_memory(&i).await.map_err(|e| e.to_string())? {
                        mem.state = MemoryState::Archived;
                        state.storage.update_memory(&mem).await.map_err(|e| e.to_string())?;
                        Ok(format!("Memory '{}' marked as archived.", i))
                    } else {
                        Err(format!("Memory '{}' not found", i))
                    }
                })
            }
            "memory_update" => {
                let id = args["id"].as_str().ok_or("Missing id")?;
                let state = self.state.clone();
                let i = id.to_string();
                let new_text = args["canonical_text"].as_str().map(|s| s.to_string());
                let pin = args["is_pinned"].as_bool();

                rt.block_on(async move {
                    if let Some(mut mem) = state.storage.get_memory(&i).await.map_err(|e| e.to_string())? {
                        if let Some(t) = new_text {
                            mem.canonical_text = t;
                        }
                        if let Some(p) = pin {
                            mem.is_pinned = p;
                        }
                        state.storage.update_memory(&mem).await.map_err(|e| e.to_string())?;
                        Ok(format!("Memory '{}' successfully updated.", i))
                    } else {
                        Err(format!("Memory '{}' not found", i))
                    }
                })
            }
            "profile_get" => {
                let container = args["container"].as_str().ok_or("Missing container")?;
                let state = self.state.clone();
                let c = container.to_string();

                rt.block_on(async move {
                    let prof = state.storage.get_profile(&c).await.map_err(|e| e.to_string())?;
                    Ok(serde_json::to_string_pretty(&prof).unwrap_or_default())
                })
            }
            "wiki_update_page" => {
                let container = args["container"].as_str().ok_or("Missing container")?;
                let state = self.state.clone();
                let c = container.to_string();

                rt.block_on(async move {
                    let sync_engine = ObsidianSyncEngine::new(state.storage.clone(), &state.vault_path);
                    let exp = sync_engine.export_vault(&c).await.map_err(|e| e.to_string())?;
                    Ok(format!("Obsidian vault synced. Exported {} pages.", exp))
                })
            }
            "project_context" => {
                let container = args["container"].as_str().ok_or("Missing container")?;
                let query = args["query"].as_str().ok_or("Missing query")?;
                let max_tokens = args["max_tokens"].as_u64().unwrap_or(2000) as usize;

                let state = self.state.clone();
                let c = container.to_string();
                let q = query.to_string();

                rt.block_on(async move {
                    let engine = HybridSearchEngine::new(state.storage.clone(), state.embedder.clone());
                    let hits = engine.search(&c, &q, 10).await.map_err(|e| e.to_string())?;
                    let prof = state.storage.get_profile(&c).await.unwrap_or(None);

                    let opts = ContextCompilerOptions {
                        max_tokens,
                        include_profile: true,
                        include_sources: true,
                    };
                    Ok(ContextCompiler::compile(prof.as_ref(), &hits, &opts))
                })
            }
            "source_get" => {
                let doc_id = args["document_id"].as_str().ok_or("Missing document_id")?;
                let state = self.state.clone();
                let d = doc_id.to_string();

                rt.block_on(async move {
                    let doc = state.storage.get_document(&d).await.map_err(|e| e.to_string())?;
                    let chunks = state.storage.get_chunks_by_document(&d).await.map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({
                        "document": doc,
                        "chunks": chunks
                    }).to_string())
                })
            }
            _ => Err(format!("Unknown or unimplemented tool: {}", name)),
        }
    }

    /// Runs stdio server loop for MCP clients
    pub fn run_stdio(&self) {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line in stdin.lock().lines() {
            if let Ok(l) = line {
                if l.trim().is_empty() {
                    continue;
                }
                if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&l) {
                    let resp = self.handle_request(req);
                    if let Ok(resp_str) = serde_json::to_string(&resp) {
                        let _ = writeln!(stdout, "{}", resp_str);
                        let _ = stdout.flush();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::local_provider::{LocalDeterministicEmbedding, LocalRuleBasedLLM};
    use crate::storage::sqlite::SqliteStorage;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_mcp_initialize_and_tools_list() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let llm = Arc::new(LocalRuleBasedLLM::new());
        let embedder = Arc::new(LocalDeterministicEmbedding::default());
        let dir = tempdir().unwrap();

        let state = Arc::new(AppState {
            storage,
            llm,
            embedder,
            vault_path: dir.path().to_path_buf(),
        });

        let server = McpServer::new(state);

        // Test initialize
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: None,
        };
        let init_resp = server.handle_request(init_req);
        assert!(init_resp.result.is_some());
        assert_eq!(init_resp.result.unwrap()["serverInfo"]["name"], "hypermemory");

        // Test tools/list
        let list_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/list".into(),
            params: None,
        };
        let list_resp = server.handle_request(list_req);
        let tools = list_resp.result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 11);
    }
}
