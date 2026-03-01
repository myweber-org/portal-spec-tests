use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_data(&mut self, data: Vec<&str>) -> Vec<String> {
        data.into_iter()
            .filter(|item| self.deduplicate(item))
            .map(|item| self.normalize_string(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["apple", "Apple", "banana", " apple ", "BANANA"];
        let cleaned = cleaner.clean_data(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  TEST  "), "test");
        assert_eq!(cleaner.normalize_string("MixedCase"), "mixedcase");
    }
}use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn process_record(&mut self, record: &str) -> Option<String> {
        let normalized = Self::normalize(record);
        
        if self.dedupe_set.contains(&normalized) {
            None
        } else {
            self.dedupe_set.insert(normalized.clone());
            Some(normalized)
        }
    }

    fn normalize(input: &str) -> String {
        input
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn batch_process(&mut self, records: Vec<&str>) -> Vec<String> {
        records
            .into_iter()
            .filter_map(|record| self.process_record(record))
            .collect()
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn get_processed_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
    if email.contains('@') && email.contains('.') {
        Ok(())
    } else {
        Err("Invalid email format".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        
        let result1 = cleaner.process_record("Hello World");
        let result2 = cleaner.process_record("hello world");
        let result3 = cleaner.process_record("New Record");
        
        assert!(result1.is_some());
        assert!(result2.is_none());
        assert!(result3.is_some());
        assert_eq!(cleaner.get_processed_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let input = "  HELLO@World!  ";
        let normalized = DataCleaner::normalize(input);
        assert_eq!(normalized, "hello world");
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let records = vec!["Test1", "test1", "Test2", "Test1"];
        
        let results = cleaner.batch_process(records);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "test1");
        assert_eq!(results[1], "test2");
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
    }
}