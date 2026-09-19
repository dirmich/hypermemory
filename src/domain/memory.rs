use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Fact,
    Preference,
    Episode,
    Decision,
    Goal,
    Task,
    Relationship,
    Inference,
    Temporary,
}

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::Fact
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    Hot,
    Warm,
    Cold,
    Archived,
    Deleted,
}

impl Default for MemoryState {
    fn default() -> Self {
        MemoryState::Hot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub id: String,
    pub container_id: String,
    pub canonical_text: String,
    pub memory_type: MemoryType,
    pub confidence: f32,
    pub importance: f32,
    pub source_id: Option<String>,
    pub source_chunk_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub last_accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub is_latest: bool,
    pub is_static: bool,
    pub is_pinned: bool,
    pub state: MemoryState,
}

impl Memory {
    pub fn new(
        container_id: impl Into<String>,
        canonical_text: impl Into<String>,
        memory_type: MemoryType,
    ) -> Self {
        let canonical_text = canonical_text.into();
        let now = Utc::now();
        let hash = blake3::hash(canonical_text.as_bytes()).to_hex().to_string();
        let id = format!("mem_{}", &hash[..16]);

        Self {
            id,
            container_id: container_id.into(),
            canonical_text,
            memory_type,
            confidence: 1.0,
            importance: 0.8,
            source_id: None,
            source_chunk_id: None,
            created_at: now,
            valid_from: now,
            valid_until: None,
            last_accessed_at: now,
            access_count: 0,
            is_latest: true,
            is_static: false,
            is_pinned: false,
            state: MemoryState::Hot,
        }
    }
}
