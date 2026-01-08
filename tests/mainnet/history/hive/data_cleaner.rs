use std::collections::HashSet;
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

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        self.dedupe_set.insert(normalized)
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_text(item));
            }
        }
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

pub fn validate_email(email: &str) -> Result<bool, Box<dyn Error>> {
    if email.is_empty() {
        return Err("Email cannot be empty".into());
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Ok(false);
    }
    
    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    Ok(domain_parts.len() >= 2 && !domain_parts.iter().any(|p| p.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
    }

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("another"));
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").unwrap());
        assert!(!validate_email("invalid-email").unwrap());
        assert!(validate_email("test@sub.domain.co.uk").unwrap());
    }
}