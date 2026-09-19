use super::hybrid::SearchHit;
use crate::domain::UserProfile;

pub struct ContextCompilerOptions {
    pub max_tokens: usize,
    pub include_profile: bool,
    pub include_sources: bool,
}

impl Default for ContextCompilerOptions {
    fn default() -> Self {
        Self {
            max_tokens: 2000,
            include_profile: true,
            include_sources: true,
        }
    }
}

pub struct ContextCompiler;

impl ContextCompiler {
    pub fn compile(
        profile: Option<&UserProfile>,
        hits: &[SearchHit],
        options: &ContextCompilerOptions,
    ) -> String {
        let mut sections = Vec::new();
        let mut estimated_tokens = 0;

        // 1. Profile section
        if options.include_profile {
            if let Some(prof) = profile {
                let mut p_text = String::from("## User Profile & Preferences\n");
                if !prof.active_projects.is_empty() {
                    p_text.push_str(&format!("- Active Projects: {}\n", prof.active_projects.join(", ")));
                }
                if !prof.preferences.is_empty() {
                    p_text.push_str(&format!("- Preferences: {}\n", prof.preferences.join(", ")));
                }
                if !prof.recent_decisions.is_empty() {
                    p_text.push_str(&format!("- Recent Decisions: {}\n", prof.recent_decisions.join(", ")));
                }

                let p_tokens = p_text.split_whitespace().count();
                if estimated_tokens + p_tokens < options.max_tokens {
                    estimated_tokens += p_tokens;
                    sections.push(p_text);
                }
            }
        }

        // 2. Active Memories section
        let mut m_text = String::from("## Relevant Current Memories\n");
        let mut added_memories = 0;

        for hit in hits {
            let line = format!(
                "- [{:?}] {} (conf: {:.2}, imp: {:.2})\n",
                hit.memory.memory_type, hit.memory.canonical_text, hit.memory.confidence, hit.memory.importance
            );
            let line_tokens = line.split_whitespace().count();
            if estimated_tokens + line_tokens >= options.max_tokens {
                break;
            }
            estimated_tokens += line_tokens;
            m_text.push_str(&line);
            added_memories += 1;
        }

        if added_memories > 0 {
            sections.push(m_text);
        }

        // 3. Sources
        if options.include_sources {
            let mut s_text = String::from("## Sources & Provenance\n");
            let mut added_sources = 0;
            for hit in hits {
                if let Some(source_id) = &hit.memory.source_id {
                    let s_line = format!("- Memory `{}` -> Source `{}`\n", hit.memory.id, source_id);
                    let s_tokens = s_line.split_whitespace().count();
                    if estimated_tokens + s_tokens < options.max_tokens {
                        estimated_tokens += s_tokens;
                        s_text.push_str(&s_line);
                        added_sources += 1;
                    }
                }
            }
            if added_sources > 0 {
                sections.push(s_text);
            }
        }

        format!(
            "# Hyper Memory Context (Budget: {} tokens, Used: ~{} tokens)\n\n{}",
            options.max_tokens,
            estimated_tokens,
            sections.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Memory, MemoryType};

    #[test]
    fn test_context_compilation_under_budget() {
        let mem = Memory::new("p1", "PostgreSQL is our database", MemoryType::Decision);
        let hit = SearchHit {
            memory: mem,
            score: 0.95,
            match_sources: vec!["hybrid".into()],
        };

        let prof = UserProfile::new("p1");
        let opts = ContextCompilerOptions {
            max_tokens: 500,
            include_profile: true,
            include_sources: true,
        };

        let output = ContextCompiler::compile(Some(&prof), &[hit], &opts);
        assert!(output.contains("PostgreSQL is our database"));
        assert!(output.contains("Budget: 500 tokens"));
    }
}
