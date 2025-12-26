use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    unique_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            unique_set: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> Result<(), Box<dyn Error>> {
        let trimmed = record.trim();
        
        if trimmed.is_empty() {
            return Err("Empty record not allowed".into());
        }

        if trimmed.len() > 1000 {
            return Err("Record exceeds maximum length".into());
        }

        if self.unique_set.contains(trimmed) {
            return Err("Duplicate record detected".into());
        }

        self.unique_set.insert(trimmed.to_string());
        self.records.push(trimmed.to_string());
        Ok(())
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_count = self.records.len();
        self.unique_set.clear();
        
        let mut deduped = Vec::new();
        for record in &self.records {
            if !self.unique_set.contains(record) {
                self.unique_set.insert(record.clone());
                deduped.push(record.clone());
            }
        }
        
        self.records = deduped;
        original_count - self.records.len()
    }

    pub fn get_records(&self) -> &[String] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.unique_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test@example.com").is_ok());
        assert_eq!(cleaner.get_records().len(), 1);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("duplicate").unwrap();
        assert!(cleaner.add_record("duplicate").is_err());
    }

    #[test]
    fn test_email_validation() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@domain.com"));
        assert!(!cleaner.validate_email("invalid-email"));
    }
}