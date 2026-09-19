use super::fts::BM25InvertedIndex;
use super::router::{QueryRouter, QueryType};
use super::vector::{VectorFormat, VectorIndex};
use crate::domain::Memory;
use crate::extractor::traits::EmbeddingProvider;
use crate::storage::traits::{StorageBackend, StorageResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SearchHit {
    pub memory: Memory,
    pub score: f32,
    pub match_sources: Vec<String>,
}

pub struct HybridSearchEngine {
    storage: Arc<dyn StorageBackend>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl HybridSearchEngine {
    pub fn new(storage: Arc<dyn StorageBackend>, embedder: Arc<dyn EmbeddingProvider>) -> Self {
        Self { storage, embedder }
    }

    pub async fn search(
        &self,
        container_id: &str,
        query: &str,
        limit: usize,
    ) -> StorageResult<Vec<SearchHit>> {
        let qtype = QueryRouter::route(query);

        // Load active memories from storage
        let memories = if qtype == QueryType::Temporal {
            self.storage.list_memories(container_id).await?
        } else {
            self.storage.list_active_memories(container_id).await?
        };

        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // Build ephemeral or cached FTS and Vector Index
        let mut fts = BM25InvertedIndex::new();
        let mut vec_index = VectorIndex::new(VectorFormat::INT8);

        for mem in &memories {
            fts.add_document(&mem.id, &mem.canonical_text);

            let cached = vec_index.get_cached_embedding("local", &mem.id);
            let emb = match cached {
                Some(v) => v,
                None => {
                    let v = self.embedder.embed(&mem.canonical_text).await.unwrap_or_default();
                    vec_index.cache_embedding("local", &mem.id, v.clone());
                    v
                }
            };
            vec_index.insert(&mem.id, &emb);
        }

        // Execute FTS
        let fts_hits = fts.search(query, memories.len());
        let max_bm25 = fts_hits.first().map(|h| h.1).unwrap_or(1.0).max(1.0);
        let mut fts_scores: HashMap<String, f32> = HashMap::new();
        for (id, s) in fts_hits {
            fts_scores.insert(id, s / max_bm25);
        }

        // Execute Vector Search
        let query_vec = self.embedder.embed(query).await.unwrap_or_default();
        let vec_hits = vec_index.search(&query_vec, memories.len());
        let mut vec_scores: HashMap<String, f32> = HashMap::new();
        for (id, s) in vec_hits {
            vec_scores.insert(id, s.max(0.0));
        }

        let now = Utc::now();
        let mut combined_hits = Vec::new();

        for mem in memories {
            let lex = *fts_scores.get(&mem.id).unwrap_or(&0.0);
            let sem = *vec_scores.get(&mem.id).unwrap_or(&0.0);

            // Recency score (decays over 30 days)
            let days_old = (now - mem.created_at).num_days().max(0) as f32;
            let recency = (-days_old / 30.0).exp();

            // Combined weighted score
            let score = 0.35 * lex + 0.35 * sem + 0.15 * recency + 0.10 * mem.importance + 0.05 * mem.confidence;

            let mut match_sources = Vec::new();
            if lex > 0.1 {
                match_sources.push("lexical".to_string());
            }
            if sem > 0.4 {
                match_sources.push("semantic".to_string());
            }

            if score > 0.15 || qtype == QueryType::Exact {
                combined_hits.push(SearchHit {
                    memory: mem,
                    score,
                    match_sources,
                });
            }
        }

        // Sort descending by score
        combined_hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        combined_hits.truncate(limit);

        Ok(combined_hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MemoryType;
    use crate::extractor::local_provider::LocalDeterministicEmbedding;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_hybrid_search() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let embedder = Arc::new(LocalDeterministicEmbedding::default());
        let engine = HybridSearchEngine::new(storage.clone(), embedder);

        let mem1 = Memory::new("proj_1", "Project uses PostgreSQL for persistent data", MemoryType::Decision);
        let mem2 = Memory::new("proj_1", "Rust provides zero cost abstractions", MemoryType::Fact);
        storage.save_memory(&mem1).await.unwrap();
        storage.save_memory(&mem2).await.unwrap();

        let hits = engine.search("proj_1", "PostgreSQL database", 5).await.unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory.id, mem1.id);
    }
}
