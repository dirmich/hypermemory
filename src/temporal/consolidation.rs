use crate::domain::{Edge, Memory, MemoryState, RelationType};
use crate::storage::traits::{StorageBackend, StorageResult};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MemoryConsolidationEngine {
    storage: Arc<dyn StorageBackend>,
    min_cluster_size: usize,
}

impl MemoryConsolidationEngine {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            storage,
            min_cluster_size: 3,
        }
    }

    pub fn with_cluster_size(storage: Arc<dyn StorageBackend>, size: usize) -> Self {
        Self {
            storage,
            min_cluster_size: size,
        }
    }

    /// Scans memories in a container and consolidates clusters of similar memories into canonical ones.
    pub async fn consolidate_memories(&self, container_id: &str) -> StorageResult<Vec<Memory>> {
        let active_memories = self.storage.list_active_memories(container_id).await?;
        let mut clusters: HashMap<String, Vec<Memory>> = HashMap::new();

        // Cluster by shared significant keywords/entities and memory type
        for mem in active_memories {
            let words: Vec<&str> = mem
                .canonical_text
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();

            for word in words {
                let key = format!("{:?}_{}", mem.memory_type, word.to_lowercase());
                clusters.entry(key).or_default().push(mem.clone());
            }
        }

        let mut consolidated_memories = Vec::new();

        for (_key, group) in clusters {
            // Deduplicate memory items in group
            let mut unique_memories: Vec<Memory> = Vec::new();
            for m in group {
                if !unique_memories.iter().any(|u| u.id == m.id) {
                    unique_memories.push(m);
                }
            }

            if unique_memories.len() >= self.min_cluster_size {
                let first = &unique_memories[0];
                let combined_text = format!(
                    "Consolidated statement regarding {}: {}",
                    _key.split('_').last().unwrap_or("topic"),
                    unique_memories
                        .iter()
                        .map(|m| m.canonical_text.as_str())
                        .collect::<Vec<&str>>()
                        .join(" | ")
                );

                let mut canonical = Memory::new(container_id, combined_text, first.memory_type);
                canonical.importance = 0.95;
                canonical.confidence = 0.98;
                self.storage.save_memory(&canonical).await?;

                // Archive original memories and link with DERIVES edges
                for mut orig in unique_memories {
                    orig.state = MemoryState::Archived;
                    self.storage.update_memory(&orig).await?;

                    let edge = Edge::new(&canonical.id, &orig.id, RelationType::Derives, 1.0);
                    self.storage.save_edge(&edge).await?;
                }

                consolidated_memories.push(canonical);
                break; // One consolidation per pass to avoid cluster overlaps
            }
        }

        Ok(consolidated_memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MemoryType;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_consolidation() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let engine = MemoryConsolidationEngine::with_cluster_size(storage.clone(), 3);

        let mem1 = Memory::new("proj_1", "User prefers Rust for performance", MemoryType::Preference);
        let mem2 = Memory::new("proj_1", "User develops Rust tools", MemoryType::Preference);
        let mem3 = Memory::new("proj_1", "User likes Rust ecosystem", MemoryType::Preference);

        storage.save_memory(&mem1).await.unwrap();
        storage.save_memory(&mem2).await.unwrap();
        storage.save_memory(&mem3).await.unwrap();

        let consolidated = engine.consolidate_memories("proj_1").await.unwrap();
        assert_eq!(consolidated.len(), 1);

        // Verify original memories are archived
        let reloaded1 = storage.get_memory(&mem1.id).await.unwrap().unwrap();
        assert_eq!(reloaded1.state, MemoryState::Archived);

        let edges = storage.get_edges_from(&consolidated[0].id).await.unwrap();
        assert_eq!(edges.len(), 3);
        assert_eq!(edges[0].relation, RelationType::Derives);
    }
}
