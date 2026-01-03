
use std::collections::HashSet;

pub struct DataCleaner {
    entries: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &str) {
        self.entries.push(entry.to_string());
    }

    pub fn clean(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = self.entries.drain(..).collect();
        let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
        unique_vec.sort();
        unique_vec
    }

    pub fn process_raw_data(raw_data: &[&str]) -> Vec<String> {
        let mut cleaner = DataCleaner::new();
        for entry in raw_data {
            cleaner.add_entry(entry);
        }
        cleaner.clean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let raw_data = vec!["apple", "banana", "apple", "cherry", "banana"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_cleaner_sorts_entries() {
        let raw_data = vec!["zebra", "apple", "mango"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "mango", "zebra"]);
    }
}