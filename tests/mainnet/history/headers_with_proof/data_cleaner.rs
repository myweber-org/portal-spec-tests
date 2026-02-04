
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.contains(&normalized) {
            return None;
        }

        self.unique_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  HELLO  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("hello"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.process_string("   "), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "BANANA", "  banana  ", "Cherry"];
        
        let result = cleaner.process_batch(&inputs);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}