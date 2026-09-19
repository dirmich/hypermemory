use crate::domain::MemoryType;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub type ExtractorResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedMemory {
    pub canonical_text: String,
    pub memory_type: MemoryType,
    pub confidence: f32,
    pub importance: f32,
    pub entities: Vec<String>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn extract_memories(&self, text: &str) -> ExtractorResult<Vec<ExtractedMemory>>;
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> ExtractorResult<Vec<f32>>;
    fn dimensions(&self) -> usize;
}
