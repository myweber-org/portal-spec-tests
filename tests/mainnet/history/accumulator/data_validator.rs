
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyString,
    InvalidEmail,
    OutOfRange { min: i32, max: i32, actual: i32 },
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyString => write!(f, "String cannot be empty"),
            ValidationError::InvalidEmail => write!(f, "Invalid email format"),
            ValidationError::OutOfRange { min, max, actual } => {
                write!(f, "Value {} is outside range [{}, {}]", actual, min, max)
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
        if email.contains('@') && email.contains('.') && email.len() > 5 {
            Ok(())
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }

    pub fn validate_range(value: i32, min: i32, max: i32) -> Result<(), ValidationError> {
        if value >= min && value <= max {
            Ok(())
        } else {
            Err(ValidationError::OutOfRange { min, max, actual: value })
        }
    }

    pub fn validate_custom<F>(value: &str, predicate: F) -> Result<(), ValidationError>
    where
        F: Fn(&str) -> bool,
    {
        if predicate(value) {
            Ok(())
        } else {
            Err(ValidationError::Custom(format!("Custom validation failed for: {}", value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_validation() {
        assert!(Validator::validate_non_empty("hello").is_ok());
        assert_eq!(Validator::validate_non_empty(""), Err(ValidationError::EmptyString));
        assert_eq!(Validator::validate_non_empty("   "), Err(ValidationError::EmptyString));
    }

    #[test]
    fn test_email_validation() {
        assert!(Validator::validate_email("test@example.com").is_ok());
        assert_eq!(Validator::validate_email("invalid"), Err(ValidationError::InvalidEmail));
        assert_eq!(Validator::validate_email("a@b.c"), Err(ValidationError::InvalidEmail));
    }

    #[test]
    fn test_range_validation() {
        assert!(Validator::validate_range(5, 1, 10).is_ok());
        assert_eq!(
            Validator::validate_range(15, 1, 10),
            Err(ValidationError::OutOfRange { min: 1, max: 10, actual: 15 })
        );
    }

    #[test]
    fn test_custom_validation() {
        let is_uppercase = |s: &str| s.chars().all(|c| c.is_uppercase());
        
        assert!(Validator::validate_custom("HELLO", &is_uppercase).is_ok());
        assert!(Validator::validate_custom("Hello", &is_uppercase).is_err());
    }
}