use super::traits::{EmbeddingProvider, ExtractedMemory, ExtractorResult, LLMProvider};
use crate::domain::MemoryType;
use async_trait::async_trait;

pub struct LocalRuleBasedLLM;

impl LocalRuleBasedLLM {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LLMProvider for LocalRuleBasedLLM {
    async fn extract_memories(&self, text: &str) -> ExtractorResult<Vec<ExtractedMemory>> {
        let mut memories = Vec::new();
        let sentences: Vec<&str> = text
            .split(['.', '\n', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for sentence in sentences {
            let lower = sentence.to_lowercase();
            let mut mem_type = MemoryType::Fact;
            let mut importance = 0.7;
            let mut entities = Vec::new();

            // Detect type
            if lower.contains("decid") || lower.contains("결정") || lower.contains("하기로") || lower.contains("바꾸") {
                mem_type = MemoryType::Decision;
                importance = 0.9;
            } else if lower.contains("prefer") || lower.contains("선호") || lower.contains("좋아") {
                mem_type = MemoryType::Preference;
                importance = 0.8;
            } else if lower.contains("goal") || lower.contains("목표") || lower.contains("예정") || lower.contains("will") {
                mem_type = MemoryType::Goal;
                importance = 0.85;
            }

            // Extract candidate entities (known tech/projects or capitalized tokens)
            let known_entities = [
                "Rust", "PostgreSQL", "ClickHouse", "Obsidian", "Hyper Memory", "SQLite",
                "Tantivy", "Python", "TypeScript", "Bun", "Tokio", "Axum", "Linux", "macOS", "Windows",
            ];
            for ke in &known_entities {
                if sentence.to_lowercase().contains(&ke.to_lowercase()) {
                    entities.push(ke.to_string());
                }
            }

            // Also check for capitalized tokens if no known entity found
            if entities.is_empty() {
                for word in sentence.split_whitespace() {
                    let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                    if let Some(first) = clean.chars().next() {
                        if first.is_uppercase() && clean.len() > 1 {
                            entities.push(clean.to_string());
                        }
                    }
                }
            }

            memories.push(ExtractedMemory {
                canonical_text: sentence.to_string(),
                memory_type: mem_type,
                confidence: 0.95,
                importance,
                entities,
            });
        }

        Ok(memories)
    }
}

pub struct LocalDeterministicEmbedding {
    dimensions: usize,
}

impl LocalDeterministicEmbedding {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Default for LocalDeterministicEmbedding {
    fn default() -> Self {
        Self { dimensions: 384 }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalDeterministicEmbedding {
    async fn embed(&self, text: &str) -> ExtractorResult<Vec<f32>> {
        let mut vector = vec![0.0f32; self.dimensions];
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        if words.is_empty() {
            return Ok(vector);
        }

        // Feature hashing into embedding dimensions
        for word in &words {
            let hash = blake3::hash(word.as_bytes());
            let bytes = hash.as_bytes();
            for i in 0..self.dimensions {
                let byte_idx = i % 32;
                let sign = if bytes[byte_idx] % 2 == 0 { 1.0 } else { -1.0 };
                let val = (bytes[(byte_idx + 1) % 32] as f32) / 255.0;
                vector[i] += sign * val;
            }
        }

        // L2 Normalization for Cosine Similarity
        let norm_sq: f32 = vector.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt();
        if norm > 0.0 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }

        Ok(vector)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_llm_extraction() {
        let llm = LocalRuleBasedLLM::new();
        let extracted = llm
            .extract_memories("We decided to use PostgreSQL for our database.")
            .await
            .unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].memory_type, MemoryType::Decision);
        assert!(extracted[0].entities.contains(&"PostgreSQL".to_string()));
    }

    #[tokio::test]
    async fn test_local_embedding() {
        let embedder = LocalDeterministicEmbedding::default();
        let v1 = embedder.embed("Rust is memory safe").await.unwrap();
        let v2 = embedder.embed("Rust is memory safe").await.unwrap();
        let v3 = embedder.embed("Cooking recipes for pasta").await.unwrap();

        assert_eq!(v1.len(), 384);
        assert_eq!(v1, v2);

        // Cosine similarity
        let dot_same: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let dot_diff: f32 = v1.iter().zip(v3.iter()).map(|(a, b)| a * b).sum();
        assert!((dot_same - 1.0).abs() < 1e-5);
        assert!(dot_diff < dot_same);
    }
}
