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

    pub fn deduplicate(&mut self, data: &str) -> Option<String> {
        if self.dedupe_set.contains(data) {
            None
        } else {
            self.dedupe_set.insert(data.to_string());
            Some(data.to_string())
        }
    }

    pub fn validate_email(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 
            && !parts[0].is_empty() 
            && !domain_parts.iter().any(|p| p.is_empty())
    }

    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn clear_cache(&mut self) {
        self.dedupe_set.clear();
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
        assert_eq!(cleaner.deduplicate("test"), Some("test".to_string()));
        assert_eq!(cleaner.deduplicate("test"), None);
        assert_eq!(cleaner.deduplicate("another"), Some("another".to_string()));
    }

    #[test]
    fn test_email_validation() {
        assert!(DataCleaner::validate_email("user@example.com"));
        assert!(!DataCleaner::validate_email("invalid-email"));
        assert!(!DataCleaner::validate_email("@domain.com"));
        assert!(!DataCleaner::validate_email("user@.com"));
    }

    #[test]
    fn test_whitespace_normalization() {
        assert_eq!(
            DataCleaner::normalize_whitespace("  multiple   spaces   here  "),
            "multiple spaces here"
        );
    }
}