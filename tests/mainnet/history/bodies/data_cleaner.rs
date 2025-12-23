
use std::collections::HashSet;

pub struct DataCleaner {
    processed_count: usize,
    duplicates_removed: usize,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            processed_count: 0,
            duplicates_removed: 0,
        }
    }

    pub fn remove_duplicates(&mut self, data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in data {
            self.processed_count += 1;
            if seen.insert(item.clone()) {
                result.push(item);
            } else {
                self.duplicates_removed += 1;
            }
        }

        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .collect()
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.processed_count, self.duplicates_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];

        let result = cleaner.remove_duplicates(data);
        assert_eq!(result.len(), 3);
        assert_eq!(cleaner.get_stats(), (4, 1));
    }

    #[test]
    fn test_normalize_strings() {
        let data = vec![
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "CHERRY".to_string(),
        ];

        let result = DataCleaner::normalize_strings(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}