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

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        !self.dedupe_set.insert(normalized)
    }

    pub fn remove_duplicates(&mut self, items: Vec<String>) -> Vec<String> {
        let mut unique_items = Vec::new();
        
        for item in items {
            if !self.is_duplicate(&item) {
                unique_items.push(item);
            }
        }
        
        unique_items
    }

    pub fn clean_numeric(&self, input: &str) -> Option<f64> {
        let cleaned: String = input.chars()
            .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
            .collect();
        
        cleaned.parse::<f64>().ok()
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
        let data = vec![
            "apple".to_string(),
            "Apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ];
        
        let result = cleaner.remove_duplicates(data);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_numeric_cleaning() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_numeric("$123.45"), Some(123.45));
        assert_eq!(cleaner.clean_numeric("invalid"), None);
    }
}