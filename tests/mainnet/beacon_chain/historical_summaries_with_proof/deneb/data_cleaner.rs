use std::collections::HashSet;

pub struct DataCleaner {
    pub data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let key: Vec<String> = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"".to_string()).to_string())
                .collect();
            seen.insert(key)
        });
    }

    pub fn clean(&mut self) {
        self.remove_null_rows();
        self.deduplicate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut raw_data = vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("C".to_string()), Some("3".to_string())],
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        cleaner.clean();

        assert_eq!(cleaner.data.len(), 2);
        assert_eq!(cleaner.data[0][0], Some("A".to_string()));
        assert_eq!(cleaner.data[1][0], Some("C".to_string()));
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
        let mut unique_set = HashSet::new();
        let mut unique_records = Vec::new();
        let mut removed_count = 0;

        for record in self.records.drain(..) {
            if unique_set.insert(record.clone()) {
                unique_records.push(record);
            } else {
                removed_count += 1;
            }
        }

        self.records = unique_records;
        removed_count
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        let mut invalid_count = 0;

        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
        }

        (valid_count, invalid_count)
    }

    pub fn get_statistics(&self) -> (usize, usize, f64) {
        let total = self.records.len();
        let total_chars: usize = self.records.iter().map(|s| s.len()).sum();
        let avg_length = if total > 0 {
            total_chars as f64 / total as f64
        } else {
            0.0
        };

        (total, total_chars, avg_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("record1".to_string());
        cleaner.add_record("record2".to_string());
        cleaner.add_record("record1".to_string());
        cleaner.add_record("record3".to_string());
        cleaner.add_record("record2".to_string());

        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 2);
        assert_eq!(cleaner.records.len(), 3);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record(&"x".repeat(1001));

        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 2);
    }

    #[test]
    fn test_statistics() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("hello".to_string());
        cleaner.add_record("world".to_string());
        cleaner.add_record("test".to_string());

        let (total, chars, avg) = cleaner.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(chars, 14);
        assert!((avg - 4.666).abs() < 0.001);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return false;
        }
        
        if self.seen.contains(&trimmed) {
            return false;
        }
        
        self.seen.insert(trimmed.clone());
        self.records.push(trimmed);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| record.len() > 3 && record.len() < 100)
            .collect()
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_len = self.records.len();
        let mut unique = Vec::new();
        let mut new_seen = HashSet::new();
        
        for record in &self.records {
            if new_seen.insert(record.clone()) {
                unique.push(record.clone());
            }
        }
        
        self.records = unique;
        self.seen = new_seen;
        original_len - self.records.len()
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test");
        cleaner.add_record("test");
        cleaner.add_record("other");
        
        assert_eq!(cleaner.get_records().len(), 2);
        assert_eq!(cleaner.deduplicate(), 0);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("abc");
        cleaner.add_record("valid_record");
        cleaner.add_record("x");
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
    }
}