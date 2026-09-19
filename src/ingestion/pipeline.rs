use super::chunker::{ChunkerOptions, ContentChunker};
use super::normalizer::ContentNormalizer;
use crate::domain::{Document, SourceType};
use crate::storage::traits::{StorageBackend, StorageResult};
use std::sync::Arc;

pub struct IngestionResult {
    pub document: Document,
    pub is_duplicate: bool,
    pub chunk_count: usize,
}

pub struct IngestionPipeline {
    storage: Arc<dyn StorageBackend>,
    chunker_options: ChunkerOptions,
}

impl IngestionPipeline {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            storage,
            chunker_options: ChunkerOptions::default(),
        }
    }

    pub fn with_options(storage: Arc<dyn StorageBackend>, options: ChunkerOptions) -> Self {
        Self {
            storage,
            chunker_options: options,
        }
    }

    pub async fn ingest_text(
        &self,
        container_id: &str,
        text: &str,
        source_type: SourceType,
        title: Option<String>,
        source_uri: Option<String>,
    ) -> StorageResult<IngestionResult> {
        let normalized = ContentNormalizer::normalize(text);
        let hash = blake3::hash(normalized.as_bytes()).to_hex().to_string();

        // Check if document already exists by content hash (Content Fingerprint Dedup)
        if let Some(existing_doc) = self.storage.find_document_by_hash(&hash).await? {
            let chunks = self.storage.get_chunks_by_document(&existing_doc.id).await?;
            return Ok(IngestionResult {
                document: existing_doc,
                is_duplicate: true,
                chunk_count: chunks.len(),
            });
        }

        let doc = Document::new(
            container_id,
            source_type.clone(),
            &normalized,
            title,
            source_uri,
            None,
        );

        self.storage.save_document(&doc).await?;

        let chunks = if source_type == SourceType::Markdown {
            ContentChunker::chunk_markdown(&doc.id, &normalized, &self.chunker_options)
        } else {
            ContentChunker::chunk_text(&doc.id, &normalized, &self.chunker_options)
        };

        let chunk_count = chunks.len();
        for chunk in &chunks {
            self.storage.save_chunk(chunk).await?;
        }

        Ok(IngestionResult {
            document: doc,
            is_duplicate: false,
            chunk_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::SqliteStorage;

    #[tokio::test]
    async fn test_ingestion_pipeline_deduplication() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let pipeline = IngestionPipeline::new(storage);

        let res1 = pipeline
            .ingest_text(
                "user_1",
                "Rust is our core engine language.",
                SourceType::Conversation,
                Some("Chat 1".into()),
                None,
            )
            .await
            .unwrap();
        assert!(!res1.is_duplicate);
        assert_eq!(res1.chunk_count, 1);

        // Same text ingested again
        let res2 = pipeline
            .ingest_text(
                "user_1",
                "Rust is our core engine language.",
                SourceType::Conversation,
                Some("Chat 2".into()),
                None,
            )
            .await
            .unwrap();
        assert!(res2.is_duplicate);
        assert_eq!(res1.document.id, res2.document.id);
    }
}
