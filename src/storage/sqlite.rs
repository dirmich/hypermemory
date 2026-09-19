use super::traits::{StorageBackend, StorageResult};
use crate::domain::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    pub fn new_in_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.initialize_schema()?;
        Ok(storage)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        // Enable WAL mode and foreign keys for performance and data integrity
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.initialize_schema()?;
        Ok(storage)
    }

    fn initialize_schema(&self) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                container_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_uri TEXT,
                content_hash TEXT NOT NULL UNIQUE,
                title TEXT,
                mime_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                metadata TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_documents_container ON documents(container_id);
            CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(content_hash);

            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                text TEXT NOT NULL,
                token_count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_chunks_document ON chunks(document_id);
            CREATE INDEX IF NOT EXISTS idx_chunks_hash ON chunks(content_hash);

            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                container_id TEXT NOT NULL,
                canonical_text TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                confidence REAL NOT NULL,
                importance REAL NOT NULL,
                source_id TEXT,
                source_chunk_id TEXT,
                created_at TEXT NOT NULL,
                valid_from TEXT NOT NULL,
                valid_until TEXT,
                last_accessed_at TEXT NOT NULL,
                access_count INTEGER NOT NULL,
                is_latest BOOLEAN NOT NULL,
                is_static BOOLEAN NOT NULL,
                is_pinned BOOLEAN NOT NULL,
                state TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_memories_container ON memories(container_id);
            CREATE INDEX IF NOT EXISTS idx_memories_latest ON memories(container_id, is_latest);
            CREATE INDEX IF NOT EXISTS idx_memories_state ON memories(state);

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                container_id TEXT NOT NULL,
                canonical_name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                aliases TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(container_id, canonical_name)
            );
            CREATE INDEX IF NOT EXISTS idx_entities_container ON entities(container_id);

            CREATE TABLE IF NOT EXISTS edges (
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation TEXT NOT NULL,
                weight REAL NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (source_id, target_id, relation)
            );
            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);

            CREATE TABLE IF NOT EXISTS profiles (
                container_id TEXT PRIMARY KEY,
                static_profile TEXT NOT NULL,
                dynamic_profile TEXT NOT NULL,
                active_projects TEXT NOT NULL,
                preferences TEXT NOT NULL,
                recent_decisions TEXT NOT NULL,
                recent_goals TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for SqliteStorage {
    async fn save_document(&self, doc: &Document) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO documents (
                id, container_id, source_type, source_uri, content_hash, title, mime_type,
                created_at, updated_at, imported_at, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                doc.id,
                doc.container_id,
                serde_json::to_string(&doc.source_type)?,
                doc.source_uri,
                doc.content_hash,
                doc.title,
                doc.mime_type,
                doc.created_at.to_rfc3339(),
                doc.updated_at.to_rfc3339(),
                doc.imported_at.to_rfc3339(),
                serde_json::to_string(&doc.metadata)?
            ],
        )?;
        Ok(())
    }

    async fn get_document(&self, id: &str) -> StorageResult<Option<Document>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, source_type, source_uri, content_hash, title, mime_type,
                    created_at, updated_at, imported_at, metadata
             FROM documents WHERE id = ?1",
        )?;
        let doc = stmt
            .query_row(params![id], |row| {
                let source_type_str: String = row.get(2)?;
                let source_type: SourceType = serde_json::from_str(&source_type_str).unwrap_or_default();
                let created_at_str: String = row.get(7)?;
                let updated_at_str: String = row.get(8)?;
                let imported_at_str: String = row.get(9)?;
                let meta_str: String = row.get(10)?;

                Ok(Document {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    source_type,
                    source_uri: row.get(3)?,
                    content_hash: row.get(4)?,
                    title: row.get(5)?,
                    mime_type: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                    imported_at: DateTime::parse_from_rfc3339(&imported_at_str).unwrap_or_default().with_timezone(&Utc),
                    metadata: serde_json::from_str(&meta_str).unwrap_or_else(|_| serde_json::json!({})),
                })
            })
            .optional()?;
        Ok(doc)
    }

    async fn find_document_by_hash(&self, hash: &str) -> StorageResult<Option<Document>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, source_type, source_uri, content_hash, title, mime_type,
                    created_at, updated_at, imported_at, metadata
             FROM documents WHERE content_hash = ?1",
        )?;
        let doc = stmt
            .query_row(params![hash], |row| {
                let source_type_str: String = row.get(2)?;
                let source_type: SourceType = serde_json::from_str(&source_type_str).unwrap_or_default();
                let created_at_str: String = row.get(7)?;
                let updated_at_str: String = row.get(8)?;
                let imported_at_str: String = row.get(9)?;
                let meta_str: String = row.get(10)?;

                Ok(Document {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    source_type,
                    source_uri: row.get(3)?,
                    content_hash: row.get(4)?,
                    title: row.get(5)?,
                    mime_type: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                    imported_at: DateTime::parse_from_rfc3339(&imported_at_str).unwrap_or_default().with_timezone(&Utc),
                    metadata: serde_json::from_str(&meta_str).unwrap_or_else(|_| serde_json::json!({})),
                })
            })
            .optional()?;
        Ok(doc)
    }

    async fn list_documents(&self, container_id: &str) -> StorageResult<Vec<Document>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, source_type, source_uri, content_hash, title, mime_type,
                    created_at, updated_at, imported_at, metadata
             FROM documents WHERE container_id = ?1 ORDER BY created_at DESC",
        )?;
        let docs = stmt
            .query_map(params![container_id], |row| {
                let source_type_str: String = row.get(2)?;
                let source_type: SourceType = serde_json::from_str(&source_type_str).unwrap_or_default();
                let created_at_str: String = row.get(7)?;
                let updated_at_str: String = row.get(8)?;
                let imported_at_str: String = row.get(9)?;
                let meta_str: String = row.get(10)?;

                Ok(Document {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    source_type,
                    source_uri: row.get(3)?,
                    content_hash: row.get(4)?,
                    title: row.get(5)?,
                    mime_type: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                    imported_at: DateTime::parse_from_rfc3339(&imported_at_str).unwrap_or_default().with_timezone(&Utc),
                    metadata: serde_json::from_str(&meta_str).unwrap_or_else(|_| serde_json::json!({})),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(docs)
    }

    async fn save_chunk(&self, chunk: &Chunk) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO chunks (
                id, document_id, sequence, content_hash, text, token_count, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                chunk.id,
                chunk.document_id,
                chunk.sequence,
                chunk.content_hash,
                chunk.text,
                chunk.token_count as i64,
                chunk.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    async fn get_chunk(&self, id: &str) -> StorageResult<Option<Chunk>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, document_id, sequence, content_hash, text, token_count, created_at
             FROM chunks WHERE id = ?1",
        )?;
        let chunk = stmt
            .query_row(params![id], |row| {
                let created_at_str: String = row.get(6)?;
                Ok(Chunk {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    sequence: row.get(2)?,
                    content_hash: row.get(3)?,
                    text: row.get(4)?,
                    token_count: row.get::<_, i64>(5)? as usize,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(chunk)
    }

    async fn find_chunk_by_hash(&self, hash: &str) -> StorageResult<Option<Chunk>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, document_id, sequence, content_hash, text, token_count, created_at
             FROM chunks WHERE content_hash = ?1 LIMIT 1",
        )?;
        let chunk = stmt
            .query_row(params![hash], |row| {
                let created_at_str: String = row.get(6)?;
                Ok(Chunk {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    sequence: row.get(2)?,
                    content_hash: row.get(3)?,
                    text: row.get(4)?,
                    token_count: row.get::<_, i64>(5)? as usize,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(chunk)
    }

    async fn get_chunks_by_document(&self, document_id: &str) -> StorageResult<Vec<Chunk>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, document_id, sequence, content_hash, text, token_count, created_at
             FROM chunks WHERE document_id = ?1 ORDER BY sequence ASC",
        )?;
        let chunks = stmt
            .query_map(params![document_id], |row| {
                let created_at_str: String = row.get(6)?;
                Ok(Chunk {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    sequence: row.get(2)?,
                    content_hash: row.get(3)?,
                    text: row.get(4)?,
                    token_count: row.get::<_, i64>(5)? as usize,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(chunks)
    }

    async fn save_memory(&self, memory: &Memory) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO memories (
                id, container_id, canonical_text, memory_type, confidence, importance,
                source_id, source_chunk_id, created_at, valid_from, valid_until,
                last_accessed_at, access_count, is_latest, is_static, is_pinned, state
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                memory.id,
                memory.container_id,
                memory.canonical_text,
                serde_json::to_string(&memory.memory_type)?,
                memory.confidence,
                memory.importance,
                memory.source_id,
                memory.source_chunk_id,
                memory.created_at.to_rfc3339(),
                memory.valid_from.to_rfc3339(),
                memory.valid_until.map(|dt| dt.to_rfc3339()),
                memory.last_accessed_at.to_rfc3339(),
                memory.access_count,
                memory.is_latest,
                memory.is_static,
                memory.is_pinned,
                serde_json::to_string(&memory.state)?
            ],
        )?;
        Ok(())
    }

    async fn get_memory(&self, id: &str) -> StorageResult<Option<Memory>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_text, memory_type, confidence, importance,
                    source_id, source_chunk_id, created_at, valid_from, valid_until,
                    last_accessed_at, access_count, is_latest, is_static, is_pinned, state
             FROM memories WHERE id = ?1",
        )?;
        let mem = stmt
            .query_row(params![id], |row| {
                let mem_type_str: String = row.get(3)?;
                let mem_type: MemoryType = serde_json::from_str(&mem_type_str).unwrap_or_default();
                let created_at_str: String = row.get(8)?;
                let valid_from_str: String = row.get(9)?;
                let valid_until_str: Option<String> = row.get(10)?;
                let last_accessed_str: String = row.get(11)?;
                let state_str: String = row.get(16)?;
                let state: MemoryState = serde_json::from_str(&state_str).unwrap_or_default();

                Ok(Memory {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_text: row.get(2)?,
                    memory_type: mem_type,
                    confidence: row.get(4)?,
                    importance: row.get(5)?,
                    source_id: row.get(6)?,
                    source_chunk_id: row.get(7)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    valid_from: DateTime::parse_from_rfc3339(&valid_from_str).unwrap_or_default().with_timezone(&Utc),
                    valid_until: valid_until_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    last_accessed_at: DateTime::parse_from_rfc3339(&last_accessed_str).unwrap_or_default().with_timezone(&Utc),
                    access_count: row.get(12)?,
                    is_latest: row.get(13)?,
                    is_static: row.get(14)?,
                    is_pinned: row.get(15)?,
                    state,
                })
            })
            .optional()?;
        Ok(mem)
    }

    async fn update_memory(&self, memory: &Memory) -> StorageResult<()> {
        self.save_memory(memory).await
    }

    async fn delete_memory(&self, id: &str) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(())
    }

    async fn list_memories(&self, container_id: &str) -> StorageResult<Vec<Memory>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_text, memory_type, confidence, importance,
                    source_id, source_chunk_id, created_at, valid_from, valid_until,
                    last_accessed_at, access_count, is_latest, is_static, is_pinned, state
             FROM memories WHERE container_id = ?1 ORDER BY created_at DESC",
        )?;
        let mems = stmt
            .query_map(params![container_id], |row| {
                let mem_type_str: String = row.get(3)?;
                let mem_type: MemoryType = serde_json::from_str(&mem_type_str).unwrap_or_default();
                let created_at_str: String = row.get(8)?;
                let valid_from_str: String = row.get(9)?;
                let valid_until_str: Option<String> = row.get(10)?;
                let last_accessed_str: String = row.get(11)?;
                let state_str: String = row.get(16)?;
                let state: MemoryState = serde_json::from_str(&state_str).unwrap_or_default();

                Ok(Memory {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_text: row.get(2)?,
                    memory_type: mem_type,
                    confidence: row.get(4)?,
                    importance: row.get(5)?,
                    source_id: row.get(6)?,
                    source_chunk_id: row.get(7)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    valid_from: DateTime::parse_from_rfc3339(&valid_from_str).unwrap_or_default().with_timezone(&Utc),
                    valid_until: valid_until_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    last_accessed_at: DateTime::parse_from_rfc3339(&last_accessed_str).unwrap_or_default().with_timezone(&Utc),
                    access_count: row.get(12)?,
                    is_latest: row.get(13)?,
                    is_static: row.get(14)?,
                    is_pinned: row.get(15)?,
                    state,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(mems)
    }

    async fn list_active_memories(&self, container_id: &str) -> StorageResult<Vec<Memory>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_text, memory_type, confidence, importance,
                    source_id, source_chunk_id, created_at, valid_from, valid_until,
                    last_accessed_at, access_count, is_latest, is_static, is_pinned, state
             FROM memories
             WHERE container_id = ?1 AND is_latest = 1 AND state != '\"deleted\"'
             ORDER BY importance DESC, created_at DESC",
        )?;
        let mems = stmt
            .query_map(params![container_id], |row| {
                let mem_type_str: String = row.get(3)?;
                let mem_type: MemoryType = serde_json::from_str(&mem_type_str).unwrap_or_default();
                let created_at_str: String = row.get(8)?;
                let valid_from_str: String = row.get(9)?;
                let valid_until_str: Option<String> = row.get(10)?;
                let last_accessed_str: String = row.get(11)?;
                let state_str: String = row.get(16)?;
                let state: MemoryState = serde_json::from_str(&state_str).unwrap_or_default();

                Ok(Memory {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_text: row.get(2)?,
                    memory_type: mem_type,
                    confidence: row.get(4)?,
                    importance: row.get(5)?,
                    source_id: row.get(6)?,
                    source_chunk_id: row.get(7)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    valid_from: DateTime::parse_from_rfc3339(&valid_from_str).unwrap_or_default().with_timezone(&Utc),
                    valid_until: valid_until_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    last_accessed_at: DateTime::parse_from_rfc3339(&last_accessed_str).unwrap_or_default().with_timezone(&Utc),
                    access_count: row.get(12)?,
                    is_latest: row.get(13)?,
                    is_static: row.get(14)?,
                    is_pinned: row.get(15)?,
                    state,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(mems)
    }

    async fn save_entity(&self, entity: &Entity) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO entities (
                id, container_id, canonical_name, entity_type, aliases, description, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entity.id,
                entity.container_id,
                entity.canonical_name,
                serde_json::to_string(&entity.entity_type)?,
                serde_json::to_string(&entity.aliases)?,
                entity.description,
                entity.created_at.to_rfc3339(),
                entity.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    async fn get_entity(&self, id: &str) -> StorageResult<Option<Entity>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_name, entity_type, aliases, description, created_at, updated_at
             FROM entities WHERE id = ?1",
        )?;
        let ent = stmt
            .query_row(params![id], |row| {
                let type_str: String = row.get(3)?;
                let aliases_str: String = row.get(4)?;
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok(Entity {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_name: row.get(2)?,
                    entity_type: serde_json::from_str(&type_str).unwrap_or_default(),
                    aliases: serde_json::from_str(&aliases_str).unwrap_or_default(),
                    description: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(ent)
    }

    async fn find_entity_by_name(&self, container_id: &str, name: &str) -> StorageResult<Option<Entity>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_name, entity_type, aliases, description, created_at, updated_at
             FROM entities WHERE container_id = ?1 AND LOWER(canonical_name) = LOWER(?2)",
        )?;
        let ent = stmt
            .query_row(params![container_id, name], |row| {
                let type_str: String = row.get(3)?;
                let aliases_str: String = row.get(4)?;
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok(Entity {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_name: row.get(2)?,
                    entity_type: serde_json::from_str(&type_str).unwrap_or_default(),
                    aliases: serde_json::from_str(&aliases_str).unwrap_or_default(),
                    description: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(ent)
    }

    async fn list_entities(&self, container_id: &str) -> StorageResult<Vec<Entity>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_id, canonical_name, entity_type, aliases, description, created_at, updated_at
             FROM entities WHERE container_id = ?1 ORDER BY canonical_name ASC",
        )?;
        let ents = stmt
            .query_map(params![container_id], |row| {
                let type_str: String = row.get(3)?;
                let aliases_str: String = row.get(4)?;
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok(Entity {
                    id: row.get(0)?,
                    container_id: row.get(1)?,
                    canonical_name: row.get(2)?,
                    entity_type: serde_json::from_str(&type_str).unwrap_or_default(),
                    aliases: serde_json::from_str(&aliases_str).unwrap_or_default(),
                    description: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(ents)
    }

    async fn save_edge(&self, edge: &Edge) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO edges (
                source_id, target_id, relation, weight, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                edge.source_id,
                edge.target_id,
                serde_json::to_string(&edge.relation)?,
                edge.weight,
                edge.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    async fn get_edges_from(&self, source_id: &str) -> StorageResult<Vec<Edge>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id, relation, weight, created_at
             FROM edges WHERE source_id = ?1",
        )?;
        let edges = stmt
            .query_map(params![source_id], |row| {
                let rel_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                Ok(Edge {
                    source_id: row.get(0)?,
                    target_id: row.get(1)?,
                    relation: serde_json::from_str(&rel_str).unwrap_or(RelationType::Related),
                    weight: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(edges)
    }

    async fn get_edges_to(&self, target_id: &str) -> StorageResult<Vec<Edge>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id, relation, weight, created_at
             FROM edges WHERE target_id = ?1",
        )?;
        let edges = stmt
            .query_map(params![target_id], |row| {
                let rel_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                Ok(Edge {
                    source_id: row.get(0)?,
                    target_id: row.get(1)?,
                    relation: serde_json::from_str(&rel_str).unwrap_or(RelationType::Related),
                    weight: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(edges)
    }

    async fn save_profile(&self, profile: &UserProfile) -> StorageResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO profiles (
                container_id, static_profile, dynamic_profile, active_projects, preferences,
                recent_decisions, recent_goals, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                profile.container_id,
                serde_json::to_string(&profile.static_profile)?,
                serde_json::to_string(&profile.dynamic_profile)?,
                serde_json::to_string(&profile.active_projects)?,
                serde_json::to_string(&profile.preferences)?,
                serde_json::to_string(&profile.recent_decisions)?,
                serde_json::to_string(&profile.recent_goals)?,
                profile.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    async fn get_profile(&self, container_id: &str) -> StorageResult<Option<UserProfile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT container_id, static_profile, dynamic_profile, active_projects, preferences,
                    recent_decisions, recent_goals, updated_at
             FROM profiles WHERE container_id = ?1",
        )?;
        let prof = stmt
            .query_row(params![container_id], |row| {
                let static_str: String = row.get(1)?;
                let dynamic_str: String = row.get(2)?;
                let proj_str: String = row.get(3)?;
                let pref_str: String = row.get(4)?;
                let dec_str: String = row.get(5)?;
                let goal_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;

                Ok(UserProfile {
                    container_id: row.get(0)?,
                    static_profile: serde_json::from_str(&static_str).unwrap_or_default(),
                    dynamic_profile: serde_json::from_str(&dynamic_str).unwrap_or_default(),
                    active_projects: serde_json::from_str(&proj_str).unwrap_or_default(),
                    preferences: serde_json::from_str(&pref_str).unwrap_or_default(),
                    recent_decisions: serde_json::from_str(&dec_str).unwrap_or_default(),
                    recent_goals: serde_json::from_str(&goal_str).unwrap_or_default(),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str).unwrap_or_default().with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(prof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_document_and_chunk_crud() {
        let storage = SqliteStorage::new_in_memory().unwrap();
        let doc = Document::new("user_1", SourceType::Conversation, "Hello world", Some("Test".into()), None, None);
        storage.save_document(&doc).await.unwrap();

        let retrieved = storage.get_document(&doc.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, Some("Test".into()));

        let found_by_hash = storage.find_document_by_hash(&doc.content_hash).await.unwrap().unwrap();
        assert_eq!(found_by_hash.id, doc.id);

        let chunk = Chunk::new(&doc.id, 0, "Hello world segment");
        storage.save_chunk(&chunk).await.unwrap();

        let chunks = storage.get_chunks_by_document(&doc.id).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world segment");
    }

    #[tokio::test]
    async fn test_sqlite_memory_and_edge_crud() {
        let storage = SqliteStorage::new_in_memory().unwrap();
        let mem1 = Memory::new("proj_1", "Project uses Postgres", MemoryType::Decision);
        let mem2 = Memory::new("proj_1", "Project uses ClickHouse", MemoryType::Decision);
        storage.save_memory(&mem1).await.unwrap();
        storage.save_memory(&mem2).await.unwrap();

        let edge = Edge::new(&mem2.id, &mem1.id, RelationType::Updates, 1.0);
        storage.save_edge(&edge).await.unwrap();

        let edges_from = storage.get_edges_from(&mem2.id).await.unwrap();
        assert_eq!(edges_from.len(), 1);
        assert_eq!(edges_from[0].relation, RelationType::Updates);

        let active = storage.list_active_memories("proj_1").await.unwrap();
        assert_eq!(active.len(), 2);
    }
}
