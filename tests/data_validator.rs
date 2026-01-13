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

    pub fn validate_length(&self, input: &str, min: usize, max: usize) -> bool {
        let len = input.chars().count();
        len >= min && len <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(validator.validate_email("user.name+tag@domain.co.uk"));
    }

    #[test]
    fn test_invalid_email() {
        let validator = Validator::new();
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("test@.com"));
    }

    #[test]
    fn test_valid_phone() {
        let validator = Validator::new();
        assert!(validator.validate_phone("+12345678901"));
        assert!(validator.validate_phone("1234567890"));
    }

    #[test]
    fn test_length_validation() {
        let validator = Validator::new();
        assert!(validator.validate_length("hello", 3, 10));
        assert!(!validator.validate_length("hi", 3, 10));
        assert!(!validator.validate_length("verylongstring", 3, 10));
    }
}