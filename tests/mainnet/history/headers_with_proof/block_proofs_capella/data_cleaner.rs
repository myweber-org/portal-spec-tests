
use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, input: &str) -> Option<String> {
        if self.dedupe_set.contains(input) {
            None
        } else {
            self.dedupe_set.insert(input.to_string());
            Some(input.to_string())
        }
    }

    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn trim_and_lowercase(text: &str) -> String {
        text.trim().to_lowercase()
    }

    pub fn clean_pipeline(&mut self, input: &str) -> Option<String> {
        let normalized = Self::normalize_whitespace(input);
        let processed = Self::trim_and_lowercase(&normalized);
        self.deduplicate(&processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test").is_some());
        assert!(cleaner.deduplicate("test").is_none());
        assert!(cleaner.deduplicate("another").is_some());
    }

    #[test]
    fn test_normalization() {
        assert_eq!(
            DataCleaner::normalize_whitespace("  multiple   spaces   here  "),
            "multiple spaces here"
        );
    }

    #[test]
    fn test_pipeline() {
        let mut cleaner = DataCleaner::new();
        let result = cleaner.clean_pipeline("  Hello  World  ");
        assert_eq!(result, Some("hello world".to_string()));
        
        let duplicate = cleaner.clean_pipeline("  Hello  World  ");
        assert!(duplicate.is_none());
    }
}