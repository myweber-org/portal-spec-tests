
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
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_enabled: bool,
    normalization_enabled: bool,
}

impl DataCleaner {
    pub fn new(deduplication: bool, normalization: bool) -> Self {
        DataCleaner {
            deduplication_enabled: deduplication,
            normalization_enabled: normalization,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.deduplication_enabled {
            processed_data = Self::remove_duplicates(processed_data);
        }

        if self.normalization_enabled {
            processed_data = Self::normalize_entries(processed_data);
        }

        processed_data
    }

    fn remove_duplicates(data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        data.into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    fn normalize_entries(data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .map(|entry| {
                entry.trim().to_lowercase()
            })
            .collect()
    }

    pub fn validate_data(&self, data: &[String]) -> bool {
        !data.is_empty() && data.iter().all(|item| !item.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new(true, false);
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec![
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "  CHERRY  ".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned[0], "apple");
        assert_eq!(cleaned[1], "banana");
        assert_eq!(cleaned[2], "cherry");
    }

    #[test]
    fn test_validation() {
        let cleaner = DataCleaner::new(false, false);
        let valid_data = vec!["valid".to_string(), "data".to_string()];
        let invalid_data = vec!["".to_string(), "   ".to_string()];
        
        assert!(cleaner.validate_data(&valid_data));
        assert!(!cleaner.validate_data(&invalid_data));
    }
}