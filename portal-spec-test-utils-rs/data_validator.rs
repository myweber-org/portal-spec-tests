
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyString,
    InvalidEmail,
    OutOfRange { min: i32, max: i32, value: i32 },
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyString => write!(f, "String cannot be empty"),
            ValidationError::InvalidEmail => write!(f, "Invalid email format"),
            ValidationError::OutOfRange { min, max, value } => {
                write!(f, "Value {} is outside range [{}, {}]", value, min, max)
            }
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct Validator;

impl Validator {
    pub fn validate_non_empty(s: &str) -> Result<(), ValidationError> {
        if s.trim().is_empty() {
            Err(ValidationError::EmptyString)
        } else {
            Ok(())
        }
    }

    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }

    pub fn validate_range(value: i32, min: i32, max: i32) -> Result<(), ValidationError> {
        if value >= min && value <= max {
            Ok(())
        } else {
            Err(ValidationError::OutOfRange { min, max, value })
        }
    }

    pub fn validate_custom<F>(value: &str, predicate: F) -> Result<(), ValidationError>
    where
        F: Fn(&str) -> bool,
    {
        if predicate(value) {
            Ok(())
        } else {
            Err(ValidationError::Custom(format!("Validation failed for: {}", value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_validation() {
        assert!(Validator::validate_non_empty("hello").is_ok());
        assert_eq!(
            Validator::validate_non_empty(""),
            Err(ValidationError::EmptyString)
        );
        assert_eq!(
            Validator::validate_non_empty("   "),
            Err(ValidationError::EmptyString)
        );
    }

    #[test]
    fn test_email_validation() {
        assert!(Validator::validate_email("test@example.com").is_ok());
        assert_eq!(
            Validator::validate_email("invalid-email"),
            Err(ValidationError::InvalidEmail)
        );
    }

    #[test]
    fn test_range_validation() {
        assert!(Validator::validate_range(5, 1, 10).is_ok());
        assert_eq!(
            Validator::validate_range(15, 1, 10),
            Err(ValidationError::OutOfRange {
                min: 1,
                max: 10,
                value: 15
            })
        );
    }

    #[test]
    fn test_custom_validation() {
        let is_uppercase = |s: &str| s.chars().all(|c| c.is_uppercase());
        assert!(Validator::validate_custom("HELLO", is_uppercase).is_ok());
        assert!(Validator::validate_custom("Hello", is_uppercase).is_err());
    }
}
use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone_number(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(is_valid_email("alice+test@domain.org"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("@domain.com"));
    }

    #[test]
    fn test_valid_phone_numbers() {
        assert!(is_valid_phone_number("+1234567890"));
        assert!(is_valid_phone_number("1234567890"));
        assert!(is_valid_phone_number("+441234567890"));
    }

    #[test]
    fn test_invalid_phone_numbers() {
        assert!(!is_valid_phone_number("123"));
        assert!(!is_valid_phone_number("+01234567890"));
        assert!(!is_valid_phone_number("abc1234567"));
    }
}