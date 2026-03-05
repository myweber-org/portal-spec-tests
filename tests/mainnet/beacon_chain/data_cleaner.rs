
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    unique_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        self.unique_ids.insert(id.to_string())
    }

    pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
        if email.is_empty() {
            return Err("Email cannot be empty".into());
        }

        if !email.contains('@') {
            return Err("Email must contain @ symbol".into());
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email format".into());
        }

        if !parts[1].contains('.') {
            return Err("Email domain must contain a dot".into());
        }

        Ok(())
    }

    pub fn normalize_phone_number(phone: &str) -> String {
        phone
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    pub fn sanitize_input(input: &str) -> String {
        input
            .trim()
            .replace('\n', " ")
            .replace('\r', " ")
            .replace('\t', " ")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("user123"));
        assert!(!cleaner.deduplicate("user123"));
        assert!(cleaner.deduplicate("user456"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com").is_ok());
        assert!(DataCleaner::validate_email("invalid").is_err());
        assert!(DataCleaner::validate_email("").is_err());
    }

    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(
            DataCleaner::normalize_phone_number("+1 (555) 123-4567"),
            "15551234567"
        );
        assert_eq!(
            DataCleaner::normalize_phone_number("555.123.4567"),
            "5551234567"
        );
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(
            DataCleaner::sanitize_input("  hello\nworld\t"),
            "hello world"
        );
    }
}