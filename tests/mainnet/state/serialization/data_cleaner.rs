
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub validate_emails: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            validate_emails: false,
        }
    }

    pub fn deduplicate_strings(&self, input: Vec<String>) -> Vec<String> {
        if !self.remove_duplicates {
            return input;
        }

        let mut seen = HashSet::new();
        input
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn clean_email_list(&self, emails: Vec<String>) -> Vec<String> {
        let mut cleaned = self.deduplicate_strings(emails);

        if self.validate_emails {
            cleaned.retain(|email| self.is_valid_email(email));
        }

        cleaned.sort();
        cleaned
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2
            && !parts[0].is_empty()
            && !domain_parts.iter().any(|part| part.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "test@example.com".to_string(),
            "test@example.com".to_string(),
            "unique@domain.com".to_string(),
        ];

        let result = cleaner.deduplicate_strings(input);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"test@example.com".to_string()));
        assert!(result.contains(&"unique@domain.com".to_string()));
    }

    #[test]
    fn test_email_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.validate_emails = true;

        let emails = vec![
            "valid@example.com".to_string(),
            "invalid-email".to_string(),
            "another@valid.org".to_string(),
            "@missinglocal.com".to_string(),
        ];

        let result = cleaner.clean_email_list(emails);
        assert_eq!(result, vec!["another@valid.org", "valid@example.com"]);
    }
}