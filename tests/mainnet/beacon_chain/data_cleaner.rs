
use std::collections::HashSet;

pub struct DataCleaner {
    processed_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            processed_items: HashSet::new(),
        }
    }

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.processed_items.contains(&normalized) {
            true
        } else {
            self.processed_items.insert(normalized);
            false
        }
    }

    pub fn clean_data(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
            }
        }
        
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.processed_items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_duplicate_detection() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.is_duplicate("test"));
        assert!(cleaner.is_duplicate("TEST"));
        assert!(cleaner.is_duplicate("  test  "));
    }

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "  banana  ".to_string(),
            "cherry".to_string(),
        ];
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}