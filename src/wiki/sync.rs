use super::compiler::WikiCompiler;
use super::writer::VaultWriter;
use crate::domain::{Memory, MemoryState, MemoryType};
use crate::storage::traits::{StorageBackend, StorageResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SyncReport {
    pub pages_generated: usize,
    pub memories_imported_from_vault: usize,
    pub control_tags_detected: usize,
}

pub struct ObsidianSyncEngine {
    storage: Arc<dyn StorageBackend>,
    vault_writer: VaultWriter,
    vault_root: PathBuf,
}

impl ObsidianSyncEngine {
    pub fn new<P: AsRef<Path>>(storage: Arc<dyn StorageBackend>, vault_root: P) -> Self {
        let vault_writer = VaultWriter::new(vault_root.as_ref());
        Self {
            storage,
            vault_writer,
            vault_root: vault_root.as_ref().to_path_buf(),
        }
    }

    /// Forward sync: projects storage entities to Obsidian vault markdown
    pub async fn export_vault(&self, container_id: &str) -> StorageResult<usize> {
        self.vault_writer.initialize_vault()?;
        let entities = self.storage.list_entities(container_id).await?;
        let compiler = WikiCompiler::new(self.storage.clone());
        let mut count = 0;

        for ent in entities {
            let ir = compiler.compile_entity_ir(&ent).await?;
            self.vault_writer.write_page(&ir)?;
            count += 1;
        }

        Ok(count)
    }

    /// Reverse sync: detects user modifications, notes, and control tags from vault
    pub async fn import_vault_changes(&self, container_id: &str) -> StorageResult<SyncReport> {
        let mut imported_memories = 0;
        let mut control_tags = 0;

        if !self.vault_root.exists() {
            return Ok(SyncReport {
                pages_generated: 0,
                memories_imported_from_vault: 0,
                control_tags_detected: 0,
            });
        }

        let mut md_files = Vec::new();
        Self::collect_md_files(&self.vault_root, &mut md_files)?;

        for file in md_files {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Check control tags
            if content.contains("#hypermemory/no-ai") {
                control_tags += 1;
                continue;
            }

            let is_pinned = content.contains("#hypermemory/pin");
            let is_archived = content.contains("#hypermemory/archive");
            if is_pinned || is_archived {
                control_tags += 1;
            }

            // Extract User Notes section
            let notes_header = "## My Notes";
            if let Some(pos) = content.find(notes_header) {
                let user_notes = &content[pos + notes_header.len()..];
                let lines: Vec<&str> = user_notes
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect();

                for line in lines {
                    let clean = line.trim_start_matches("- ").trim();
                    if clean.len() >= 10 {
                        let mut mem = Memory::new(container_id, clean, MemoryType::Fact);
                        mem.importance = 0.85;
                        mem.confidence = 0.95;
                        mem.is_pinned = is_pinned;
                        if is_archived {
                            mem.state = MemoryState::Archived;
                        }

                        self.storage.save_memory(&mem).await?;
                        imported_memories += 1;
                    }
                }
            }
        }

        Ok(SyncReport {
            pages_generated: 0,
            memories_imported_from_vault: imported_memories,
            control_tags_detected: control_tags,
        })
    }

    fn collect_md_files(dir: &Path, list: &mut Vec<PathBuf>) -> StorageResult<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_md_files(&path, list)?;
                } else if path.extension().map_or(false, |ext| ext == "md") {
                    list.push(path);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Entity, EntityType};
    use crate::storage::sqlite::SqliteStorage;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_two_way_obsidian_sync() {
        let storage = Arc::new(SqliteStorage::new_in_memory().unwrap());
        let dir = tempdir().unwrap();
        let engine = ObsidianSyncEngine::new(storage.clone(), dir.path());

        // 1. Create entity and export to vault
        let ent = Entity::new("c1", "ClickHouse", EntityType::Technology);
        storage.save_entity(&ent).await.unwrap();

        let count = engine.export_vault("c1").await.unwrap();
        assert_eq!(count, 1);

        let tech_file = dir.path().join("Technologies").join("ClickHouse.md");
        assert!(tech_file.exists());

        // 2. User writes a note with #hypermemory/pin in Obsidian
        let initial_content = fs::read_to_string(&tech_file).unwrap();
        let modified_content = format!(
            "{}\n- ClickHouse clusters need Zookeeper or Keeper. #hypermemory/pin",
            initial_content
        );
        fs::write(&tech_file, modified_content).unwrap();

        // 3. Reverse sync import
        let report = engine.import_vault_changes("c1").await.unwrap();
        assert!(report.memories_imported_from_vault >= 1);
        assert!(report.control_tags_detected >= 1);

        let active_mems = storage.list_active_memories("c1").await.unwrap();
        assert!(active_mems.iter().any(|m| m.canonical_text.contains("Zookeeper")));
    }
}
