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

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty() 
                && record.len() <= 100 
                && record.chars().all(|c| c.is_ascii())
            })
            .collect()
    }

    pub fn clean_all(&mut self) -> (usize, Vec<bool>) {
        let duplicates_removed = self.remove_duplicates();
        let validation_results = self.validate_records();
        (duplicates_removed, validation_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validate_records() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("a".repeat(101));
        
        let results = cleaner.validate_records();
        assert_eq!(results, vec![true, false, false]);
    }
}