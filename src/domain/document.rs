use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Conversation,
    File,
    Markdown,
    Codebase,
    Note,
    Custom(String),
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Conversation
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: String,
    pub container_id: String,
    pub source_type: SourceType,
    pub source_uri: Option<String>,
    pub content_hash: String,
    pub title: Option<String>,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub imported_at: DateTime<Utc>,
    pub metadata: Value,
}

impl Document {
    pub fn new(
        container_id: impl Into<String>,
        source_type: SourceType,
        content: &str,
        title: Option<String>,
        source_uri: Option<String>,
        metadata: Option<Value>,
    ) -> Self {
        let now = Utc::now();
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let id = format!("doc_{}", &content_hash[..16]);

        Self {
            id,
            container_id: container_id.into(),
            source_type,
            source_uri,
            content_hash,
            title,
            mime_type: "text/plain".to_string(),
            created_at: now,
            updated_at: now,
            imported_at: now,
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
        }
    }
}
