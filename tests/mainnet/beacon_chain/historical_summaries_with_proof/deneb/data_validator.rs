
use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_user_data(email: &str, phone: &str) -> Result<(), String> {
    if !is_valid_email(email) {
        return Err(format!("Invalid email address: {}", email));
    }
    
    if !is_valid_phone(phone) {
        return Err(format!("Invalid phone number: {}", phone));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@.com"));
    }

    #[test]
    fn test_valid_phone() {
        assert!(is_valid_phone("+1234567890"));
        assert!(is_valid_phone("1234567890"));
        assert!(!is_valid_phone("abc"));
        assert!(!is_valid_phone("123"));
    }

    #[test]
    fn test_validate_user_data() {
        assert!(validate_user_data("test@example.com", "+1234567890").is_ok());
        assert!(validate_user_data("invalid", "+1234567890").is_err());
        assert!(validate_user_data("test@example.com", "invalid").is_err());
    }
}