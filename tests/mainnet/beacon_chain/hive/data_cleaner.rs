
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

    pub fn process(&mut self, input: &str) -> Option<String> {
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
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process("Hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("  HELLO  "), None);
        assert_eq!(cleaner.process("world"), Some("world".to_string()));
        assert_eq!(cleaner.process(""), None);
        
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_clear_functionality() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.process("test");
        assert_eq!(cleaner.get_unique_count(), 1);
        
        cleaner.clear();
        assert_eq!(cleaner.get_unique_count(), 0);
        
        assert_eq!(cleaner.process("test"), Some("test".to_string()));
    }
}