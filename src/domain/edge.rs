use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationType {
    Updates,
    Extends,
    Derives,
    Related,
    Contradicts,
    Supports,
    Mentions,
    BelongsTo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub relation: RelationType,
    pub weight: f32,
    pub created_at: DateTime<Utc>,
}

impl Edge {
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relation: RelationType,
        weight: f32,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            relation,
            weight,
            created_at: Utc::now(),
        }
    }
}
