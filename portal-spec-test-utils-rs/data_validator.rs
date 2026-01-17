use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_phone_number(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_string_length(input: &str, min: usize, max: usize) -> bool {
    let len = input.chars().count();
    len >= min && len <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("john.doe@company.co.uk"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("user@.com"));
    }

    #[test]
    fn test_valid_phone_number() {
        assert!(validate_phone_number("+1234567890"));
        assert!(validate_phone_number("1234567890"));
        assert!(!validate_phone_number("abc123"));
        assert!(!validate_phone_number("123"));
    }

    #[test]
    fn test_string_length() {
        assert!(validate_string_length("hello", 3, 10));
        assert!(!validate_string_length("hi", 3, 10));
        assert!(!validate_string_length("verylongstring", 3, 10));
    }
}