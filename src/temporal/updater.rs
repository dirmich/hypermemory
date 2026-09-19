use crate::domain::{Edge, Memory, RelationType};
use crate::storage::traits::{StorageBackend, StorageResult};
use chrono::Utc;
use std::sync::Arc;

pub struct TemporalUpdateEngine {
    storage: Arc<dyn StorageBackend>,
}

impl TemporalUpdateEngine {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }

    /// Checks if the new memory updates, extends, or contradicts any existing memories in the container.
    /// Returns the updated memory and any supersession edges created.
    pub async fn process_temporal_relations(
        &self,
        new_memory: &mut Memory,
    ) -> StorageResult<Vec<Edge>> {
        let active_memories = self
            .storage
            .list_active_memories(&new_memory.container_id)
            .await?;

        let mut created_edges = Vec::new();
        let new_text_lower = new_memory.canonical_text.to_lowercase();

        for mut old_memory in active_memories {
            if old_memory.id == new_memory.id {
                continue;
            }

            let old_text_lower = old_memory.canonical_text.to_lowercase();

            // Detection of supersession / update:
            // e.g., both talk about database choice or technology preference, and new indicates a change
            let is_replacement = (new_text_lower.contains("instead of") && new_text_lower.contains(&old_text_lower))
                || (new_text_lower.contains("대신") || new_text_lower.contains("바꾸") || new_text_lower.contains("switched to") || new_text_lower.contains("moved to"))
                || (old_memory.memory_type == new_memory.memory_type
                    && (new_text_lower.contains("use") && old_text_lower.contains("use"))
                    && old_text_lower != new_text_lower);

            let is_contradiction = (old_text_lower.contains("prefer") && new_text_lower.contains("not prefer"))
                || (old_text_lower.contains("선호") && new_text_lower.contains("선호하지 않"));

            if is_replacement {
                // Supersession: old memory becomes non-latest
                let now = Utc::now();
                old_memory.is_latest = false;
                old_memory.valid_until = Some(now);
                self.storage.update_memory(&old_memory).await?;

                let edge = Edge::new(&new_memory.id, &old_memory.id, RelationType::Updates, 1.0);
                self.storage.save_edge(&edge).await?;
                created_edges.push(edge);
            } else if is_contradiction {
                let edge = Edge::new(
                    &new_memory.id,
                    &old_memory.id,
                    RelationType::Contradicts,
                    0.9,
                );
                self.storage.save_edge(&edge).await?;
                created_edges.push(edge);
            } else if old_memory.memory_type == new_memory.memory_type {
                // Check for semantic extension / support
                let new_words: Vec<&str> = new_text_lower.split_whitespace().collect();
                let matching_words = new_words
                    .iter()
                    .filter(|w| old_text_lower.contains(*w) && w.len() > 3)
                    .count();

                if matching_words >= 2 {
                    let edge = Edge::new(&new_memory.id, &old_memory.id, RelationType::Extends, 0.7);
                    self.storage.save_edge(&edge).await?;
                    created_edges.push(edge);
                }
            }
        }

        Ok(created_edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MemoryType;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_fact_supersession() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let engine = TemporalUpdateEngine::new(storage.clone());

        let old_mem = Memory::new("proj_1", "Project uses PostgreSQL for DB", MemoryType::Decision);
        storage.save_memory(&old_mem).await.unwrap();

        let mut new_mem = Memory::new(
            "proj_1",
            "Project switched to ClickHouse instead of PostgreSQL",
            MemoryType::Decision,
        );
        storage.save_memory(&new_mem).await.unwrap();

        let edges = engine.process_temporal_relations(&mut new_mem).await.unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation, RelationType::Updates);

        // Verify old memory has been superseded
        let reloaded_old = storage.get_memory(&old_mem.id).await.unwrap().unwrap();
        assert!(!reloaded_old.is_latest);
        assert!(reloaded_old.valid_until.is_some());
    }
}
