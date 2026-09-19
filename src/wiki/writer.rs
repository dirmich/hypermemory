use super::compiler::WikiCompiler;
use super::ir::WikiPageIR;
use crate::storage::traits::StorageResult;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct VaultWriter {
    vault_root: PathBuf,
}

impl VaultWriter {
    pub fn new<P: AsRef<Path>>(vault_root: P) -> Self {
        Self {
            vault_root: vault_root.as_ref().to_path_buf(),
        }
    }

    pub fn initialize_vault(&self) -> StorageResult<()> {
        let folders = [
            "People",
            "Projects",
            "Technologies",
            "Decisions",
            "Topics",
            "Daily",
            "Sources",
            "System",
        ];

        for f in &folders {
            let path = self.vault_root.join(f);
            fs::create_dir_all(path)?;
        }

        // Initialize Home.md if not exists
        let home_path = self.vault_root.join("Home.md");
        if !home_path.exists() {
            let home_content = "# Hyper Memory Vault\n\nWelcome to your local-first AI memory and living wiki.\n\n## Folders\n- [[Projects/]]\n- [[Technologies/]]\n- [[Decisions/]]\n- [[People/]]\n- [[Daily/]]\n- [[Sources/]]\n";
            Self::atomic_write(&home_path, home_content)?;
        }

        Ok(())
    }

    pub fn write_page(&self, ir: &WikiPageIR) -> StorageResult<PathBuf> {
        let folder_path = self.vault_root.join(&ir.folder);
        fs::create_dir_all(&folder_path)?;

        let filename = format!("{}.md", ir.title.replace(['/', '\\', ':', '*'], "_"));
        let target_file = folder_path.join(filename);

        let existing_content = if target_file.exists() {
            fs::read_to_string(&target_file).ok()
        } else {
            None
        };

        let rendered = WikiCompiler::render_markdown(ir, existing_content.as_deref());
        Self::atomic_write(&target_file, &rendered)?;

        Ok(target_file)
    }

    /// Cross-platform atomic file writer using temporary file and atomic swap
    pub fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> StorageResult<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_file_path = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        {
            let mut file = fs::File::create(&temp_file_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(&temp_file_path, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EntityType;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_vault_writer_and_atomic_write() {
        let dir = tempdir().unwrap();
        let writer = VaultWriter::new(dir.path());
        writer.initialize_vault().unwrap();

        assert!(dir.path().join("Home.md").exists());
        assert!(dir.path().join("Technologies").is_dir());

        let ir = WikiPageIR {
            id: "ent_rust".into(),
            title: "Rust Language".into(),
            page_type: EntityType::Technology,
            folder: "Technologies".into(),
            summary: "Systems language".into(),
            facts: vec!["Memory safe".into()],
            decisions: vec![],
            related_pages: vec![],
            sources: vec![],
            updated_at: Utc::now(),
        };

        let written_path = writer.write_page(&ir).unwrap();
        assert!(written_path.exists());

        let content = fs::read_to_string(written_path).unwrap();
        assert!(content.contains("Rust Language"));
        assert!(content.contains("Memory safe"));
    }
}
