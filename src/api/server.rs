use super::handlers::*;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/documents", post(ingest_document_handler))
        .route("/v1/search", post(search_handler))
        .route("/v1/context", post(context_handler))
        .route("/v1/memories", get(list_memories_handler))
        .route("/v1/profile/{container}", get(get_profile_handler))
        .route("/v1/wiki/sync", post(wiki_sync_handler))
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::local_provider::{LocalDeterministicEmbedding, LocalRuleBasedLLM};
    use crate::storage::sqlite::SqliteStorage;
    use axum::body::Body;
    use axum::http::Request;
    use tempfile::tempdir;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_api_health_and_ingest() {
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

        let app = create_router(state);

        // Health check
        let req = Request::builder().uri("/health").body(Body::empty()).unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), 200);

        // Ingest Document
        let payload = serde_json::json!({
            "container": "api_test",
            "content": "We decided to deploy ClickHouse on AWS.",
            "title": "Cloud Decision"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/v1/documents")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), 200);
    }
}
