
use regex::Regex;
use std::collections::HashSet;

pub struct InputValidator {
    email_regex: Regex,
    forbidden_words: HashSet<String>,
}

impl InputValidator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let forbidden = HashSet::from([
            "malicious".to_string(),
            "injection".to_string(),
            "script".to_string(),
        ]);

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

    pub fn check_length(&self, text: &str, min: usize, max: usize) -> bool {
        let len = text.trim().len();
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
        let input = "This contains malicious script injection";
        let output = validator.sanitize_text(input);
        assert!(!output.contains("malicious"));
        assert!(!output.contains("injection"));
    }

    #[test]
    fn test_length_validation() {
        let validator = InputValidator::new();
        assert!(validator.check_length("Hello", 3, 10));
        assert!(!validator.check_length("Hi", 3, 10));
    }
}
use regex::Regex;

pub struct Validator {
    email_regex: Regex,
    phone_regex: Regex,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
            phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap(),
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_phone(&self, phone: &str) -> bool {
        self.phone_regex.is_match(phone)
    }

    pub fn validate_all(&self, email: &str, phone: &str) -> (bool, bool) {
        (self.validate_email(email), self.validate_phone(phone))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = Validator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(validator.validate_email("test.user+tag@domain.co.uk"));
    }

    #[test]
    fn test_invalid_email() {
        let validator = Validator::new();
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("user@.com"));
    }

    #[test]
    fn test_valid_phone() {
        let validator = Validator::new();
        assert!(validator.validate_phone("+12345678901"));
        assert!(validator.validate_phone("1234567890"));
    }

    #[test]
    fn test_invalid_phone() {
        let validator = Validator::new();
        assert!(!validator.validate_phone("abc"));
        assert!(!validator.validate_phone("123"));
    }
}