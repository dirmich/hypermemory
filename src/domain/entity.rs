use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Project,
    Organization,
    Technology,
    Place,
    Concept,
    Document,
    Event,
    Product,
    Custom,
}

impl Default for EntityType {
    fn default() -> Self {
        EntityType::Concept
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: String,
    pub container_id: String,
    pub canonical_name: String,
    pub entity_type: EntityType,
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    pub fn new(
        container_id: impl Into<String>,
        canonical_name: impl Into<String>,
        entity_type: EntityType,
    ) -> Self {
        let canonical_name = canonical_name.into();
        let now = Utc::now();
        let hash = blake3::hash(canonical_name.to_lowercase().as_bytes())
            .to_hex()
            .to_string();
        let id = format!("ent_{}", &hash[..16]);

        Self {
            id,
            container_id: container_id.into(),
            canonical_name,
            entity_type,
            aliases: Vec::new(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}
