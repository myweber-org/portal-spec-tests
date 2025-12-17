
use regex::Regex;
use std::collections::HashSet;

pub struct InputValidator {
    email_regex: Regex,
    forbidden_words: HashSet<String>,
}

impl InputValidator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let mut forbidden = HashSet::new();
        forbidden.insert("malicious".to_string());
        forbidden.insert("injection".to_string());
        forbidden.insert("exploit".to_string());

        InputValidator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_words: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email.trim())
    }

    pub fn sanitize_text(&self, text: &str) -> String {
        let mut sanitized = text.to_string();
        for word in &self.forbidden_words {
            let replacement = "*".repeat(word.len());
            sanitized = sanitized.replace(word, &replacement);
        }
        sanitized.trim().to_string()
    }

    pub fn check_length(&self, input: &str, min: usize, max: usize) -> bool {
        let len = input.trim().len();
        len >= min && len <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = InputValidator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_sanitization() {
        let validator = InputValidator::new();
        let result = validator.sanitize_text("This contains malicious code");
        assert!(!result.contains("malicious"));
        assert!(result.contains("*********"));
    }

    #[test]
    fn test_length_validation() {
        let validator = InputValidator::new();
        assert!(validator.check_length("hello", 3, 10));
        assert!(!validator.check_length("hi", 3, 10));
    }
}