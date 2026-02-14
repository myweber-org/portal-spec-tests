
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
}use std::collections::HashSet;

pub fn clean_strings(strings: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();
    
    for s in strings {
        let normalized = s.trim().to_lowercase();
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            cleaned.push(normalized);
        }
    }
    
    cleaned.sort();
    cleaned
}

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "hello".to_string(),
            "WORLD".to_string(),
            "world ".to_string(),
            "".to_string(),
            "  ".to_string(),
        ];
        
        let result = clean_strings(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_remove_duplicates() {
        let input = vec![1, 2, 2, 3, 1, 4, 5, 4];
        let result = remove_duplicates(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
}