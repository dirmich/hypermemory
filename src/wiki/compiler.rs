use super::ir::WikiPageIR;
use crate::domain::{Entity, MemoryType};
use crate::storage::traits::{StorageBackend, StorageResult};
use chrono::Utc;
use std::sync::Arc;

pub struct WikiCompiler {
    storage: Arc<dyn StorageBackend>,
}

impl WikiCompiler {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }

    pub async fn compile_entity_ir(&self, entity: &Entity) -> StorageResult<WikiPageIR> {
        let memories = self.storage.list_active_memories(&entity.container_id).await?;
        let edges = self.storage.get_edges_to(&entity.id).await?;

        let mut facts = Vec::new();
        let mut decisions = Vec::new();
        let mut related_pages = Vec::new();
        let mut sources = Vec::new();

        // Check which memories mention this entity
        for mem in memories {
            let mentions_entity = edges.iter().any(|e| e.source_id == mem.id)
                || mem.canonical_text.to_lowercase().contains(&entity.canonical_name.to_lowercase());

            if mentions_entity {
                if mem.memory_type == MemoryType::Decision {
                    decisions.push(mem.canonical_text.clone());
                } else {
                    facts.push(mem.canonical_text.clone());
                }

                if let Some(src) = mem.source_id {
                    if !sources.contains(&src) {
                        sources.push(src);
                    }
                }
            }
        }

        // Find connected entities for Wikilinks
        let out_edges = self.storage.get_edges_from(&entity.id).await?;
        for e in out_edges {
            if let Some(target_ent) = self.storage.get_entity(&e.target_id).await? {
                let link = format!("[[{}]]", target_ent.canonical_name);
                if !related_pages.contains(&link) {
                    related_pages.push(link);
                }
            }
        }

        let summary = entity
            .description
            .clone()
            .unwrap_or_else(|| format!("Documentation and facts for {}.", entity.canonical_name));

        Ok(WikiPageIR {
            id: entity.id.clone(),
            title: entity.canonical_name.clone(),
            page_type: entity.entity_type,
            folder: WikiPageIR::folder_for_type(entity.entity_type).to_string(),
            summary,
            facts,
            decisions,
            related_pages,
            sources,
            updated_at: Utc::now(),
        })
    }

    pub fn render_markdown(ir: &WikiPageIR, existing_content: Option<&str>) -> String {
        let mut auto_section = String::new();
        auto_section.push_str("<!-- HYPER_MEMORY:AUTO:BEGIN -->\n");
        auto_section.push_str("## Summary\n");
        auto_section.push_str(&format!("{}\n\n", ir.summary));

        if !ir.facts.is_empty() {
            auto_section.push_str("## Facts & Status\n");
            for f in &ir.facts {
                auto_section.push_str(&format!("- {}\n", f));
            }
            auto_section.push('\n');
        }

        if !ir.decisions.is_empty() {
            auto_section.push_str("## Decisions\n");
            for d in &ir.decisions {
                auto_section.push_str(&format!("- {}\n", d));
            }
            auto_section.push('\n');
        }

        if !ir.related_pages.is_empty() {
            auto_section.push_str("## Related\n");
            for r in &ir.related_pages {
                auto_section.push_str(&format!("- {}\n", r));
            }
            auto_section.push('\n');
        }

        if !ir.sources.is_empty() {
            auto_section.push_str("## Sources\n");
            for s in &ir.sources {
                auto_section.push_str(&format!("- [[Sources/{}]]\n", s));
            }
            auto_section.push('\n');
        }

        auto_section.push_str("<!-- HYPER_MEMORY:AUTO:END -->");

        // Merge with existing user content outside AUTO block
        if let Some(existing) = existing_content {
            let auto_start = "<!-- HYPER_MEMORY:AUTO:BEGIN -->";
            let auto_end = "<!-- HYPER_MEMORY:AUTO:END -->";

            if let (Some(start_idx), Some(end_idx)) = (existing.find(auto_start), existing.find(auto_end)) {
                let prefix = &existing[..start_idx];
                let suffix = &existing[end_idx + auto_end.len()..];
                return format!("{}{}\n{}", prefix, auto_section, suffix.trim_start());
            }
        }

        // New page format
        format!(
            "---\nhyper_memory_id: {}\ntype: {:?}\nupdated: {}\nmanaged: true\n---\n\n# {}\n\n{}\n\n## My Notes\n\n",
            ir.id,
            ir.page_type,
            ir.updated_at.format("%Y-%m-%d"),
            ir.title,
            auto_section
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EntityType;

    #[test]
    fn test_render_preserves_user_notes() {
        let ir = WikiPageIR {
            id: "ent_1".into(),
            title: "PostgreSQL".into(),
            page_type: EntityType::Technology,
            folder: "Technologies".into(),
            summary: "Relational database".into(),
            facts: vec!["Supports JSONB".into()],
            decisions: vec!["Primary DB".into()],
            related_pages: vec!["[[Rust]]".into()],
            sources: vec!["src_1".into()],
            updated_at: Utc::now(),
        };

        let initial_rendered = WikiCompiler::render_markdown(&ir, None);
        assert!(initial_rendered.contains("## My Notes"));

        // User edits their notes
        let edited_by_user = format!("{}\nCustom personal comment about tuning autovacuum.", initial_rendered);

        // Recompile with updated IR
        let mut updated_ir = ir.clone();
        updated_ir.facts.push("Version 16 in use".into());

        let recompiled = WikiCompiler::render_markdown(&updated_ir, Some(&edited_by_user));

        assert!(recompiled.contains("Version 16 in use"));
        assert!(recompiled.contains("Custom personal comment about tuning autovacuum."));
    }
}
