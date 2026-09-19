use crate::domain::{Memory, MemoryState, MemoryType};
use crate::storage::traits::{StorageBackend, StorageResult};
use chrono::Utc;
use std::sync::Arc;

pub struct MemoryLifecycleManager {
    storage: Arc<dyn StorageBackend>,
    hot_days: i64,
    warm_days: i64,
}

impl MemoryLifecycleManager {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            storage,
            hot_days: 30,
            warm_days: 365,
        }
    }

    pub fn with_thresholds(storage: Arc<dyn StorageBackend>, hot_days: i64, warm_days: i64) -> Self {
        Self {
            storage,
            hot_days,
            warm_days,
        }
    }

    /// Evaluates tier migration and decay for a given memory.
    pub async fn evaluate_memory(&self, memory: &mut Memory) -> StorageResult<bool> {
        let mut modified = false;
        let now = Utc::now();
        let age_days = (now - memory.created_at).num_days();

        // Pinned memories are always Hot
        if memory.is_pinned {
            if memory.state != MemoryState::Hot {
                memory.state = MemoryState::Hot;
                modified = true;
            }
            return if modified {
                self.storage.update_memory(memory).await?;
                Ok(true)
            } else {
                Ok(false)
            };
        }

        // Tier transitions based on age and access
        let new_state = if age_days < self.hot_days || memory.access_count > 10 {
            MemoryState::Hot
        } else if age_days < self.warm_days {
            MemoryState::Warm
        } else {
            MemoryState::Cold
        };

        if memory.state != new_state && memory.state != MemoryState::Archived && memory.state != MemoryState::Deleted {
            memory.state = new_state;
            modified = true;
        }

        // Decay logic for temporary and episodic memories
        if memory.memory_type == MemoryType::Temporary || memory.memory_type == MemoryType::Episode {
            let last_access_days = (now - memory.last_accessed_at).num_days();
            if last_access_days > 7 {
                memory.importance *= 0.9;
                modified = true;
                if memory.importance < 0.2 {
                    memory.state = MemoryState::Archived;
                }
            }
        }

        if modified {
            self.storage.update_memory(memory).await?;
        }

        Ok(modified)
    }

    /// Records an access event for a memory, potentially promoting its state.
    pub async fn record_access(&self, memory_id: &str) -> StorageResult<Option<Memory>> {
        if let Some(mut memory) = self.storage.get_memory(memory_id).await? {
            memory.access_count += 1;
            memory.last_accessed_at = Utc::now();

            // Promotion to Hot if accessed frequently or retrieved from Cold/Warm
            if memory.state == MemoryState::Cold || memory.state == MemoryState::Warm {
                memory.state = MemoryState::Hot;
            }

            self.storage.update_memory(&memory).await?;
            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_lifecycle_and_promotion() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let manager = MemoryLifecycleManager::new(storage.clone());

        let mut mem = Memory::new("proj_1", "Rust is great", MemoryType::Preference);
        mem.state = MemoryState::Cold;
        storage.save_memory(&mem).await.unwrap();

        // Recording access promotes to Hot
        let updated = manager.record_access(&mem.id).await.unwrap().unwrap();
        assert_eq!(updated.state, MemoryState::Hot);
        assert_eq!(updated.access_count, 1);
    }
}
