use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorFormat {
    FP32,
    INT8,
}

#[derive(Debug, Clone)]
pub enum StoredVector {
    FP32(Vec<f32>),
    INT8 { values: Vec<i8>, scale: f32 },
}

impl StoredVector {
    pub fn from_f32_slice(slice: &[f32], format: VectorFormat) -> Self {
        match format {
            VectorFormat::FP32 => StoredVector::FP32(slice.to_vec()),
            VectorFormat::INT8 => {
                let max_abs = slice.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
                let scale = if max_abs > 0.0 { max_abs / 127.0 } else { 1.0 };
                let values: Vec<i8> = slice
                    .iter()
                    .map(|v| (v / scale).round().clamp(-128.0, 127.0) as i8)
                    .collect();
                StoredVector::INT8 { values, scale }
            }
        }
    }

    pub fn cosine_similarity(&self, query_f32: &[f32]) -> f32 {
        match self {
            StoredVector::FP32(vec) => {
                let dot: f32 = vec.iter().zip(query_f32.iter()).map(|(a, b)| a * b).sum();
                let norm_a: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
                let norm_b: f32 = query_f32.iter().map(|v| v * v).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
            StoredVector::INT8 { values, scale } => {
                let mut dot: f32 = 0.0;
                for (val, q) in values.iter().zip(query_f32.iter()) {
                    let dequantized = (*val as f32) * scale;
                    dot += dequantized * q;
                }
                let norm_a: f32 = values
                    .iter()
                    .map(|v| {
                        let f = (*v as f32) * scale;
                        f * f
                    })
                    .sum::<f32>()
                    .sqrt();
                let norm_b: f32 = query_f32.iter().map(|v| v * v).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
        }
    }
}

pub struct VectorIndex {
    vectors: HashMap<String, StoredVector>,
    embedding_cache: HashMap<String, Vec<f32>>,
    format: VectorFormat,
}

impl VectorIndex {
    pub fn new(format: VectorFormat) -> Self {
        Self {
            vectors: HashMap::new(),
            embedding_cache: HashMap::new(),
            format,
        }
    }

    pub fn insert(&mut self, id: &str, vector: &[f32]) {
        let stored = StoredVector::from_f32_slice(vector, self.format);
        self.vectors.insert(id.to_string(), stored);
    }

    pub fn search(&self, query_vector: &[f32], limit: usize) -> Vec<(String, f32)> {
        let mut results: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, stored)| (id.clone(), stored.cosine_similarity(query_vector)))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    pub fn get_cached_embedding(&self, model_id: &str, content_hash: &str) -> Option<Vec<f32>> {
        let key = format!("{}_{}", model_id, content_hash);
        self.embedding_cache.get(&key).cloned()
    }

    pub fn cache_embedding(&mut self, model_id: &str, content_hash: &str, vector: Vec<f32>) {
        let key = format!("{}_{}", model_id, content_hash);
        self.embedding_cache.insert(key, vector);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_search_fp32_and_int8() {
        let v1 = vec![1.0, 0.0, 0.0, 0.0]; // Exact match with query direction
        let v2 = vec![0.0, 1.0, 0.0, 0.0]; // Orthogonal
        let v3 = vec![0.5, 0.5, 0.0, 0.0]; // Partial match

        // FP32 test
        let mut idx_fp32 = VectorIndex::new(VectorFormat::FP32);
        idx_fp32.insert("item1", &v1);
        idx_fp32.insert("item2", &v2);
        idx_fp32.insert("item3", &v3);

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let hits = idx_fp32.search(&query, 2);
        assert_eq!(hits[0].0, "item1");

        // INT8 test
        let mut idx_int8 = VectorIndex::new(VectorFormat::INT8);
        idx_int8.insert("item1", &v1);
        idx_int8.insert("item2", &v2);
        idx_int8.insert("item3", &v3);

        let hits_int8 = idx_int8.search(&query, 2);
        assert_eq!(hits_int8[0].0, "item1");
    }
}
