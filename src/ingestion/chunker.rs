use crate::domain::Chunk;

pub struct ChunkerOptions {
    pub max_chars: usize,
    pub overlap_chars: usize,
}

impl Default for ChunkerOptions {
    fn default() -> Self {
        Self {
            max_chars: 800,
            overlap_chars: 150,
        }
    }
}

pub struct ContentChunker;

impl ContentChunker {
    pub fn chunk_text(document_id: &str, text: &str, options: &ChunkerOptions) -> Vec<Chunk> {
        let text = text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        if text.len() <= options.max_chars {
            return vec![Chunk::new(document_id, 0, text)];
        }

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_len = chars.len();
        let mut start = 0;
        let mut sequence = 0;

        while start < total_len {
            let mut end = (start + options.max_chars).min(total_len);

            // Attempt to break at whitespace or punctuation if not at the very end
            if end < total_len {
                let mut candidate = end;
                while candidate > start + (options.max_chars / 2) {
                    if chars[candidate - 1].is_whitespace() || chars[candidate - 1] == '.' || chars[candidate - 1] == '\n' {
                        end = candidate;
                        break;
                    }
                    candidate -= 1;
                }
            }

            let slice: String = chars[start..end].iter().collect();
            let trimmed = slice.trim();
            if !trimmed.is_empty() {
                chunks.push(Chunk::new(document_id, sequence, trimmed));
                sequence += 1;
            }

            if end >= total_len {
                break;
            }

            let next_start = end.saturating_sub(options.overlap_chars);
            if next_start <= start {
                start = end;
            } else {
                start = next_start;
            }
        }

        chunks
    }

    pub fn chunk_markdown(document_id: &str, text: &str, options: &ChunkerOptions) -> Vec<Chunk> {
        // Markdown-aware chunking: prioritize section headers
        let lines: Vec<&str> = text.lines().collect();
        let mut sections = Vec::new();
        let mut current_section = String::new();

        for line in lines {
            if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
                if !current_section.trim().is_empty() {
                    sections.push(current_section.trim().to_string());
                    current_section.clear();
                }
            }
            current_section.push_str(line);
            current_section.push('\n');
        }
        if !current_section.trim().is_empty() {
            sections.push(current_section.trim().to_string());
        }

        let mut chunks = Vec::new();
        let mut sequence = 0;

        for section in sections {
            if section.len() <= options.max_chars {
                chunks.push(Chunk::new(document_id, sequence, section));
                sequence += 1;
            } else {
                let sub_chunks = Self::chunk_text(document_id, &section, options);
                for mut sc in sub_chunks {
                    sc.sequence = sequence;
                    chunks.push(sc);
                    sequence += 1;
                }
            }
        }

        if chunks.is_empty() && !text.trim().is_empty() {
            Self::chunk_text(document_id, text, options)
        } else {
            chunks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_chunking() {
        let text = "Hyper Memory is a local-first memory engine. ".repeat(30);
        let options = ChunkerOptions {
            max_chars: 150,
            overlap_chars: 30,
        };
        let chunks = ContentChunker::chunk_text("doc_1", &text, &options);
        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].document_id, "doc_1");
        assert_eq!(chunks[0].sequence, 0);
        assert_eq!(chunks[1].sequence, 1);
    }

    #[test]
    fn test_markdown_chunking() {
        let md = "# Section 1\nContent for section 1.\n\n## Section 2\nContent for section 2.";
        let options = ChunkerOptions::default();
        let chunks = ContentChunker::chunk_markdown("doc_2", md, &options);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].text.contains("Section 1"));
        assert!(chunks[1].text.contains("Section 2"));
    }
}
