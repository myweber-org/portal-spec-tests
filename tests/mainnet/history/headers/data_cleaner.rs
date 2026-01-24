use std::collections::HashSet;

pub struct DataCleaner {
    entries: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            entries: HashSet::new(),
        }
    }

    pub fn add_entry(&mut self, input: &str) -> bool {
        let normalized = input.trim().to_lowercase();
        self.entries.insert(normalized)
    }

    pub fn get_unique_entries(&self) -> Vec<String> {
        let mut result: Vec<String> = self.entries.iter().cloned().collect();
        result.sort();
        result
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_entry("Apple");
        cleaner.add_entry("apple ");
        cleaner.add_entry("Banana");

        assert_eq!(cleaner.count(), 2);
        let entries = cleaner.get_unique_entries();
        assert_eq!(entries, vec!["apple", "banana"]);
    }

    #[test]
    fn test_clear_function() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_entry("Test");
        cleaner.clear();
        assert_eq!(cleaner.count(), 0);
    }
}