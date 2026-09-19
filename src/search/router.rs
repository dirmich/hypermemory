#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Profile,
    Exact,
    Lexical,
    Temporal,
    Hybrid,
}

pub struct QueryRouter;

impl QueryRouter {
    pub fn route(query: &str) -> QueryType {
        let trimmed = query.trim();
        let lower = trimmed.to_lowercase();

        // Check for Profile queries
        let profile_keywords = [
            "who am i", "my name", "내 이름", "내 프로필", "나에 대해", "내 정보", "preferences", "내 선호",
        ];
        for pk in &profile_keywords {
            if lower.contains(pk) {
                return QueryType::Profile;
            }
        }

        // Check for exact lookup (e.g. enclosed in quotes)
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
            return QueryType::Exact;
        }

        // Check for temporal query
        let temporal_keywords = [
            "when", "earlier", "history", "timeline", "before", "최근", "이전", "전에", "히스토리", "과거", "변천",
        ];
        for tk in &temporal_keywords {
            if lower.contains(tk) {
                return QueryType::Temporal;
            }
        }

        // Check for short single-term lexical lookup
        if !trimmed.contains(' ') && trimmed.len() > 2 {
            return QueryType::Lexical;
        }

        QueryType::Hybrid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_routing() {
        assert_eq!(QueryRouter::route("내 이름이 뭐야?"), QueryType::Profile);
        assert_eq!(QueryRouter::route("\"PostgreSQL\""), QueryType::Exact);
        assert_eq!(QueryRouter::route("전에 어떤 결정을 했지?"), QueryType::Temporal);
        assert_eq!(QueryRouter::route("ClickHouse"), QueryType::Lexical);
        assert_eq!(
            QueryRouter::route("What database should we use for analytics?"),
            QueryType::Hybrid
        );
    }
}
