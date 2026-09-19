use crate::domain::*;
use async_trait::async_trait;
use std::error::Error;

pub type StorageResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    // Document operations
    async fn save_document(&self, doc: &Document) -> StorageResult<()>;
    async fn get_document(&self, id: &str) -> StorageResult<Option<Document>>;
    async fn find_document_by_hash(&self, hash: &str) -> StorageResult<Option<Document>>;
    async fn list_documents(&self, container_id: &str) -> StorageResult<Vec<Document>>;

    // Chunk operations
    async fn save_chunk(&self, chunk: &Chunk) -> StorageResult<()>;
    async fn get_chunk(&self, id: &str) -> StorageResult<Option<Chunk>>;
    async fn find_chunk_by_hash(&self, hash: &str) -> StorageResult<Option<Chunk>>;
    async fn get_chunks_by_document(&self, document_id: &str) -> StorageResult<Vec<Chunk>>;

    // Memory operations
    async fn save_memory(&self, memory: &Memory) -> StorageResult<()>;
    async fn get_memory(&self, id: &str) -> StorageResult<Option<Memory>>;
    async fn update_memory(&self, memory: &Memory) -> StorageResult<()>;
    async fn delete_memory(&self, id: &str) -> StorageResult<()>;
    async fn list_memories(&self, container_id: &str) -> StorageResult<Vec<Memory>>;
    async fn list_active_memories(&self, container_id: &str) -> StorageResult<Vec<Memory>>;

    // Entity operations
    async fn save_entity(&self, entity: &Entity) -> StorageResult<()>;
    async fn get_entity(&self, id: &str) -> StorageResult<Option<Entity>>;
    async fn find_entity_by_name(&self, container_id: &str, name: &str) -> StorageResult<Option<Entity>>;
    async fn list_entities(&self, container_id: &str) -> StorageResult<Vec<Entity>>;

    // Graph Edge operations
    async fn save_edge(&self, edge: &Edge) -> StorageResult<()>;
    async fn get_edges_from(&self, source_id: &str) -> StorageResult<Vec<Edge>>;
    async fn get_edges_to(&self, target_id: &str) -> StorageResult<Vec<Edge>>;

    // Profile operations
    async fn save_profile(&self, profile: &UserProfile) -> StorageResult<()>;
    async fn get_profile(&self, container_id: &str) -> StorageResult<Option<UserProfile>>;
}
