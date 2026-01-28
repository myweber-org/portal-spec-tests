use regex::Regex;
use std::collections::HashSet;

pub struct InputValidator {
    email_regex: Regex,
    forbidden_words: HashSet<String>,
}

impl InputValidator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let forbidden = vec![
            "script", "javascript", "onload", "onerror", "eval"
        ].into_iter().map(String::from).collect();

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
    fn test_email_validation() {
        let validator = InputValidator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_sanitization() {
        let validator = InputValidator::new();
        let input = "Hello<script>alert('xss')</script>";
        let output = validator.sanitize_text(input);
        assert!(!output.contains("script"));
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

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '@' || *c == '.')
            .collect()
    }

    pub fn validate_length(&self, input: &str, min: usize, max: usize) -> bool {
        let len = input.chars().count();
        len >= min && len <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_phone_validation() {
        let validator = Validator::new();
        assert!(validator.validate_phone("+1234567890"));
        assert!(!validator.validate_phone("abc"));
    }

    #[test]
    fn test_sanitize_input() {
        let validator = Validator::new();
        let result = validator.sanitize_input("Hello<script>alert('xss')</script>");
        assert_eq!(result, "Helloscriptalertxssscript");
    }

    #[test]
    fn test_length_validation() {
        let validator = Validator::new();
        assert!(validator.validate_length("test", 1, 10));
        assert!(!validator.validate_length("", 1, 10));
    }
}use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let digits_only: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    digits_only.len() >= 10 && digits_only.len() <= 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("missing@domain"));
    }

    #[test]
    fn test_valid_phones() {
        assert!(is_valid_phone("+1 (555) 123-4567"));
        assert!(is_valid_phone("5551234567"));
    }

    #[test]
    fn test_invalid_phones() {
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone(""));
    }
}