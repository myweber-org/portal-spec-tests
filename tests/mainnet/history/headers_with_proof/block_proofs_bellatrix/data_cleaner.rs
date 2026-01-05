
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
use std::collections::HashSet;

pub struct DataCleaner {
    pub records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let original_len = self.records.len();
        let mut seen = HashSet::new();
        
        self.records.retain(|record| {
            if seen.contains(record) {
                false
            } else {
                seen.insert(record.clone());
                true
            }
        });
        
        original_len - self.records.len()
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        
        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            }
        }
        
        (valid_count, self.records.len() - valid_count)
    }

    pub fn clean_all(&mut self) -> (usize, usize) {
        let duplicates_removed = self.remove_duplicates();
        let (valid, invalid) = self.validate_records();
        
        self.records.retain(|record| {
            !record.trim().is_empty() && record.len() <= 1000
        });
        
        (duplicates_removed, invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("x".repeat(1001));
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 2);
    }
}