
use std::collections::HashSet;

pub struct DataCleaner {
    seen_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            seen_items: HashSet::new(),
        }
    }

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.seen_items.contains(&normalized) {
            return None;
        }

        self.seen_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn unique_count(&self) -> usize {
        self.seen_items.len()
    }

    pub fn reset(&mut self) {
        self.seen_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  Hello  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("HELLO"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.process_string("   "), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "Banana", "  banana  ", "Cherry"];
        
        let results = cleaner.process_batch(&inputs);
        assert_eq!(results.len(), 3);
        assert_eq!(cleaner.unique_count(), 3);
    }
}