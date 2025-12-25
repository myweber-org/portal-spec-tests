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

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .filter(|item| self.deduplicate(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(!cleaner.deduplicate("  test  "));
    }

    #[test]
    fn test_dataset_cleaning() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "  banana  ".to_string(),
            "cherry".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}