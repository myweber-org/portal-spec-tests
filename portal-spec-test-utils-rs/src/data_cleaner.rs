
use regex::Regex;

pub fn clean_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_alphanumeric() {
        assert_eq!(clean_alphanumeric("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_alphanumeric("Test@#$%^&*()String"), "TestString");
        assert_eq!(clean_alphanumeric("123_456-789"), "123456789");
        assert_eq!(clean_alphanumeric(""), "");
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub deduplicate: bool,
    pub validate_email: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplicate: true,
            validate_email: false,
        }
    }

    pub fn clean_records(&self, records: Vec<String>) -> Vec<String> {
        let mut cleaned = records;

        if self.deduplicate {
            cleaned = self.remove_duplicates(cleaned);
        }

        if self.validate_email {
            cleaned = self.filter_valid_emails(cleaned);
        }

        cleaned
    }

    fn remove_duplicates(&self, records: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        records
            .into_iter()
            .filter(|record| seen.insert(record.clone()))
            .collect()
    }

    fn filter_valid_emails(&self, records: Vec<String>) -> Vec<String> {
        records
            .into_iter()
            .filter(|record| record.contains('@') && record.contains('.'))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "test@example.com".to_string(),
            "test@example.com".to_string(),
            "unique@domain.com".to_string(),
        ];
        
        let result = cleaner.clean_records(data);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_email_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.validate_email = true;
        
        let data = vec![
            "valid@email.com".to_string(),
            "invalid-email".to_string(),
            "another@test.org".to_string(),
        ];
        
        let result = cleaner.clean_records(data);
        assert_eq!(result.len(), 2);
    }
}