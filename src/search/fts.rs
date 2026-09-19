use std::collections::HashMap;

pub struct BM25Document {
    pub id: String,
    pub terms: Vec<String>,
    pub length: usize,
}

pub struct BM25InvertedIndex {
    doc_count: usize,
    avg_doc_length: f32,
    doc_lengths: HashMap<String, usize>,
    inverted_index: HashMap<String, Vec<(String, f32)>>, // term -> [(doc_id, tf)]
    k1: f32,
    b: f32,
}

impl BM25InvertedIndex {
    pub fn new() -> Self {
        Self {
            doc_count: 0,
            avg_doc_length: 0.0,
            doc_lengths: HashMap::new(),
            inverted_index: HashMap::new(),
            k1: 1.2,
            b: 0.75,
        }
    }

    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    pub fn add_document(&mut self, id: &str, text: &str) {
        let tokens = Self::tokenize(text);
        let len = tokens.len();
        if len == 0 {
            return;
        }

        // Count term frequency in this document
        let mut tf_map: HashMap<String, usize> = HashMap::new();
        for t in &tokens {
            *tf_map.entry(t.clone()).or_insert(0) += 1;
        }

        for (term, count) in tf_map {
            self.inverted_index
                .entry(term)
                .or_default()
                .push((id.to_string(), count as f32));
        }

        self.doc_lengths.insert(id.to_string(), len);
        self.doc_count += 1;
        let total_length: usize = self.doc_lengths.values().sum();
        self.avg_doc_length = total_length as f32 / self.doc_count as f32;
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<(String, f32)> {
        let query_tokens = Self::tokenize(query);
        if query_tokens.is_empty() || self.doc_count == 0 {
            return Vec::new();
        }

        let mut doc_scores: HashMap<String, f32> = HashMap::new();

        for term in query_tokens {
            if let Some(postings) = self.inverted_index.get(&term) {
                let n_q = postings.len() as f32;
                // Standard BM25 IDF
                let idf = ((self.doc_count as f32 - n_q + 0.5) / (n_q + 0.5) + 1.0).ln();

                for (doc_id, tf) in postings {
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&1) as f32;
                    let denom = tf + self.k1 * (1.0 - self.b + self.b * (doc_len / self.avg_doc_length.max(1.0)));
                    let term_score = idf * (tf * (self.k1 + 1.0)) / denom.max(1e-6);

                    *doc_scores.entry(doc_id.clone()).or_insert(0.0) += term_score;
                }
            }
        }

        let mut results: Vec<(String, f32)> = doc_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_search() {
        let mut index = BM25InvertedIndex::new();
        index.add_document("doc1", "PostgreSQL is a powerful relational database");
        index.add_document("doc2", "ClickHouse is an ultra fast columnar analytical database");
        index.add_document("doc3", "Rust is a systems programming language with zero-cost abstractions");

        let results = index.search("PostgreSQL relational", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "doc1");

        let rust_results = index.search("Rust language", 10);
        assert!(!rust_results.is_empty());
        assert_eq!(rust_results[0].0, "doc3");
    }
}
