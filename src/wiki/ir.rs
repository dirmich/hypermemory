use crate::domain::EntityType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiPageIR {
    pub id: String,
    pub title: String,
    pub page_type: EntityType,
    pub folder: String,
    pub summary: String,
    pub facts: Vec<String>,
    pub decisions: Vec<String>,
    pub related_pages: Vec<String>,
    pub sources: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl WikiPageIR {
    pub fn folder_for_type(entity_type: EntityType) -> &'static str {
        match entity_type {
            EntityType::Person => "People",
            EntityType::Project => "Projects",
            EntityType::Technology => "Technologies",
            EntityType::Organization => "Organizations",
            EntityType::Event => "Daily",
            _ => "Topics",
        }
    }
}
