use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_length(input: &str, min: usize, max: usize) -> bool {
    let len = input.chars().count();
    len >= min && len <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name+tag@domain.co.uk"));
    }

    #[test]
    fn test_invalid_email() {
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
    }

    #[test]
    fn test_valid_phone() {
        assert!(validate_phone("+12345678901"));
        assert!(validate_phone("1234567890"));
    }

    #[test]
    fn test_length_validation() {
        assert!(validate_length("hello", 3, 10));
        assert!(!validate_length("hi", 3, 10));
        assert!(!validate_length("verylongstring", 3, 10));
    }
}