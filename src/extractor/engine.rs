use super::filter::CheapNoiseFilter;
use super::traits::{ExtractorResult, LLMProvider};
use crate::domain::{Edge, Entity, EntityType, Memory, RelationType};
use crate::storage::traits::StorageBackend;
use std::sync::Arc;

pub struct MemoryExtractionEngine {
    storage: Arc<dyn StorageBackend>,
    llm: Arc<dyn LLMProvider>,
}

impl MemoryExtractionEngine {
    pub fn new(storage: Arc<dyn StorageBackend>, llm: Arc<dyn LLMProvider>) -> Self {
        Self { storage, llm }
    }

    pub async fn extract_and_store(
        &self,
        container_id: &str,
        text: &str,
        source_id: Option<String>,
        source_chunk_id: Option<String>,
    ) -> ExtractorResult<Vec<Memory>> {
        if !CheapNoiseFilter::is_memory_candidate(text) {
            return Ok(Vec::new());
        }

        let extracted = self.llm.extract_memories(text).await?;
        let mut created_memories = Vec::new();

        for item in extracted {
            let mut mem = Memory::new(container_id, item.canonical_text, item.memory_type);
            mem.confidence = item.confidence;
            mem.importance = item.importance;
            mem.source_id = source_id.clone();
            mem.source_chunk_id = source_chunk_id.clone();

            self.storage.save_memory(&mem).await?;

            // Resolve entities
            for ent_name in item.entities {
                let entity = match self.storage.find_entity_by_name(container_id, &ent_name).await? {
                    Some(existing) => existing,
                    None => {
                        let new_ent = Entity::new(container_id, &ent_name, EntityType::Concept);
                        self.storage.save_entity(&new_ent).await?;
                        new_ent
                    }
                };

                // Link memory with entity: Memory MENTIONS Entity
                let edge = Edge::new(&mem.id, &entity.id, RelationType::Mentions, 1.0);
                self.storage.save_edge(&edge).await?;
            }

            created_memories.push(mem);
        }

        Ok(created_memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::local_provider::LocalRuleBasedLLM;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_extraction_engine() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let llm = Arc::new(LocalRuleBasedLLM::new());
        let engine = MemoryExtractionEngine::new(storage.clone(), llm);

        let text = "We decided to use Rust for Hyper Memory.";
        let memories = engine
            .extract_and_store("proj_x", text, Some("doc_1".into()), None)
            .await
            .unwrap();

        assert!(!memories.is_empty());
        assert_eq!(memories[0].container_id, "proj_x");

        let edges = storage.get_edges_from(&memories[0].id).await.unwrap();
        assert!(!edges.is_empty());
    }
}
