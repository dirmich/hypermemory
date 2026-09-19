use crate::domain::{Memory, MemoryType, SourceType, UserProfile};
use crate::extractor::engine::MemoryExtractionEngine;
use crate::extractor::traits::{EmbeddingProvider, LLMProvider};
use crate::ingestion::pipeline::IngestionPipeline;
use crate::search::context_compiler::{ContextCompiler, ContextCompilerOptions};
use crate::search::hybrid::HybridSearchEngine;
use crate::storage::traits::StorageBackend;
use crate::temporal::updater::TemporalUpdateEngine;
use crate::wiki::sync::ObsidianSyncEngine;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub storage: Arc<dyn StorageBackend>,
    pub llm: Arc<dyn LLMProvider>,
    pub embedder: Arc<dyn EmbeddingProvider>,
    pub vault_path: PathBuf,
}

#[derive(Deserialize)]
pub struct IngestDocumentRequest {
    pub content: String,
    pub container: String,
    pub source_type: Option<SourceType>,
    pub title: Option<String>,
    pub source_uri: Option<String>,
}

#[derive(Serialize)]
pub struct IngestDocumentResponse {
    pub document_id: String,
    pub is_duplicate: bool,
    pub chunk_count: usize,
    pub extracted_memories: usize,
}

pub async fn ingest_document_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestDocumentRequest>,
) -> Result<Json<IngestDocumentResponse>, (StatusCode, String)> {
    let pipeline = IngestionPipeline::new(state.storage.clone());
    let source_type = payload.source_type.unwrap_or(SourceType::Conversation);

    let res = pipeline
        .ingest_text(
            &payload.container,
            &payload.content,
            source_type,
            payload.title,
            payload.source_uri,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let extraction_engine = MemoryExtractionEngine::new(state.storage.clone(), state.llm.clone());
    let mut memories = extraction_engine
        .extract_and_store(
            &payload.container,
            &payload.content,
            Some(res.document.id.clone()),
            None,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let temporal_engine = TemporalUpdateEngine::new(state.storage.clone());
    for mem in &mut memories {
        let _ = temporal_engine.process_temporal_relations(mem).await;
    }

    Ok(Json(IngestDocumentResponse {
        document_id: res.document.id,
        is_duplicate: res.is_duplicate,
        chunk_count: res.chunk_count,
        extracted_memories: memories.len(),
    }))
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub container: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub hits: Vec<SearchHitDto>,
}

#[derive(Serialize)]
pub struct SearchHitDto {
    pub id: String,
    pub text: String,
    pub memory_type: MemoryType,
    pub score: f32,
    pub match_sources: Vec<String>,
}

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let engine = HybridSearchEngine::new(state.storage.clone(), state.embedder.clone());
    let hits = engine
        .search(&payload.container, &payload.query, payload.limit.unwrap_or(10))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let dtos = hits
        .into_iter()
        .map(|h| SearchHitDto {
            id: h.memory.id,
            text: h.memory.canonical_text,
            memory_type: h.memory.memory_type,
            score: h.score,
            match_sources: h.match_sources,
        })
        .collect();

    Ok(Json(SearchResponse {
        query: payload.query,
        hits: dtos,
    }))
}

#[derive(Deserialize)]
pub struct ContextRequest {
    pub query: String,
    pub container: String,
    pub max_tokens: Option<usize>,
}

#[derive(Serialize)]
pub struct ContextResponse {
    pub context: String,
    pub query: String,
}

pub async fn context_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContextRequest>,
) -> Result<Json<ContextResponse>, (StatusCode, String)> {
    let engine = HybridSearchEngine::new(state.storage.clone(), state.embedder.clone());
    let hits = engine
        .search(&payload.container, &payload.query, 15)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let profile = state
        .storage
        .get_profile(&payload.container)
        .await
        .unwrap_or(None);

    let opts = ContextCompilerOptions {
        max_tokens: payload.max_tokens.unwrap_or(2000),
        include_profile: true,
        include_sources: true,
    };

    let compiled = ContextCompiler::compile(profile.as_ref(), &hits, &opts);

    Ok(Json(ContextResponse {
        context: compiled,
        query: payload.query,
    }))
}

#[derive(Deserialize)]
pub struct ContainerQuery {
    pub container: String,
}

pub async fn list_memories_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ContainerQuery>,
) -> Result<Json<Vec<Memory>>, (StatusCode, String)> {
    let mems = state
        .storage
        .list_memories(&query.container)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(mems))
}

pub async fn get_profile_handler(
    State(state): State<Arc<AppState>>,
    Path(container): Path<String>,
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    let prof = state
        .storage
        .get_profile(&container)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| UserProfile::new(container));
    Ok(Json(prof))
}

#[derive(Deserialize)]
pub struct WikiSyncRequest {
    pub container: String,
}

#[derive(Serialize)]
pub struct WikiSyncResponse {
    pub exported_pages: usize,
    pub imported_memories: usize,
}

pub async fn wiki_sync_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WikiSyncRequest>,
) -> Result<Json<WikiSyncResponse>, (StatusCode, String)> {
    let sync_engine = ObsidianSyncEngine::new(state.storage.clone(), &state.vault_path);
    let exported = sync_engine
        .export_vault(&payload.container)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let import_report = sync_engine
        .import_vault_changes(&payload.container)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(WikiSyncResponse {
        exported_pages: exported,
        imported_memories: import_report.memories_imported_from_vault,
    }))
}

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
