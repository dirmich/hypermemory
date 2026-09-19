pub struct CheapNoiseFilter;

impl CheapNoiseFilter {
    /// Determines if a text contains substantial memory-worthy information
    /// without incurring expensive LLM calls.
    pub fn is_memory_candidate(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.len() < 4 {
            return false;
        }

        let lower = trimmed.to_lowercase();
        let common_chatter = [
            "응", "네", "ㅋㅋ", "ㅎㅎ", "고마워", "감사합니다", "알겠어", "알겠습니다", "오케이", "좋아",
            "yes", "no", "ok", "okay", "thanks", "thank you", "cool", "sure", "got it", "yep", "nope", "lol",
        ];

        for chatter in &common_chatter {
            if lower == *chatter || lower == format!("{}.", chatter) || lower == format!("{}!", chatter) {
                return false;
            }
        }

        // Check for indicators of facts, decisions, preferences, or entities
        let indicators = [
            // Korean indicators
            "사용", "결정", "선호", "좋아", "개발", "프로젝트", "계획", "이사", "이전", "바꾸", "하자", "버전", "DB",
            // English indicators
            "use", "using", "decided", "prefer", "likes", "building", "project", "plan", "moved", "switch", "will", "database", "is a", "are",
        ];

        for ind in &indicators {
            if lower.contains(ind) {
                return true;
            }
        }

        // Substantive sentence check: at least 15 characters and multiple words
        trimmed.len() >= 15 && trimmed.split_whitespace().count() >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_filtering() {
        assert!(!CheapNoiseFilter::is_memory_candidate("응"));
        assert!(!CheapNoiseFilter::is_memory_candidate("thanks"));
        assert!(!CheapNoiseFilter::is_memory_candidate("ok!"));
        assert!(!CheapNoiseFilter::is_memory_candidate("ㅋㅋ"));

        assert!(CheapNoiseFilter::is_memory_candidate("나는 새 프로젝트에서 PostgreSQL을 사용하기로 했어."));
        assert!(CheapNoiseFilter::is_memory_candidate("We decided to use Rust for the core engine."));
    }
}
