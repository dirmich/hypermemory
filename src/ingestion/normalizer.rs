use unicode_normalization::UnicodeNormalization;

pub struct ContentNormalizer;

impl ContentNormalizer {
    /// Normalizes input text using Unicode NFKC, collapses consecutive whitespaces,
    /// and trims leading and trailing whitespace.
    pub fn normalize(input: &str) -> String {
        let nfkc = input.nfkc().collect::<String>();
        let mut result = String::with_capacity(nfkc.len());
        let mut in_whitespace = false;

        for ch in nfkc.chars() {
            if ch.is_whitespace() {
                if !in_whitespace {
                    result.push(' ');
                    in_whitespace = true;
                }
            } else {
                result.push(ch);
                in_whitespace = false;
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_normalization() {
        let raw = "   Hello \t\t  world! \n\n This   is  HyperMemory.   ";
        let normalized = ContentNormalizer::normalize(raw);
        assert_eq!(normalized, "Hello world! This is HyperMemory.");
    }
}
