use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub sequence: u32,
    pub content_hash: String,
    pub text: String,
    pub token_count: usize,
    pub created_at: DateTime<Utc>,
}

impl Chunk {
    pub fn new(document_id: impl Into<String>, sequence: u32, text: impl Into<String>) -> Self {
        let text = text.into();
        let content_hash = blake3::hash(text.as_bytes()).to_hex().to_string();
        let id = format!("chk_{}", &content_hash[..16]);
        let token_count = text.split_whitespace().count(); // approximate token count

        Self {
            id,
            document_id: document_id.into(),
            sequence,
            content_hash,
            text,
            token_count,
            created_at: Utc::now(),
        }
    }
}
