
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
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(validator.validate_email("user.name@domain.co.uk"));
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("test@.com"));
    }

    #[test]
    fn test_phone_validation() {
        let validator = Validator::new();
        assert!(validator.validate_phone("+1234567890"));
        assert!(validator.validate_phone("1234567890"));
        assert!(!validator.validate_phone("abc123"));
        assert!(!validator.validate_phone("123"));
    }
}