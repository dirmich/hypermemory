use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub container_id: String,
    pub static_profile: Value,
    pub dynamic_profile: Value,
    pub active_projects: Vec<String>,
    pub preferences: Vec<String>,
    pub recent_decisions: Vec<String>,
    pub recent_goals: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl UserProfile {
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            container_id: container_id.into(),
            static_profile: serde_json::json!({}),
            dynamic_profile: serde_json::json!({}),
            active_projects: Vec::new(),
            preferences: Vec::new(),
            recent_decisions: Vec::new(),
            recent_goals: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}
