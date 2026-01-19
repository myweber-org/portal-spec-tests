
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    EmptyField,
    InvalidFormat,
    OutOfRange,
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField => write!(f, "Field cannot be empty"),
            ValidationError::InvalidFormat => write!(f, "Invalid data format"),
            ValidationError::OutOfRange => write!(f, "Value out of acceptable range"),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct Validator;

impl Validator {
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if email.is_empty() {
            return Err(ValidationError::EmptyField);
        }

        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidFormat);
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidFormat);
        }

        Ok(())
    }

    pub fn validate_age(age: u8) -> Result<(), ValidationError> {
        if age < 18 {
            return Err(ValidationError::OutOfRange);
        }

        if age > 120 {
            return Err(ValidationError::Custom(String::from("Age exceeds reasonable limit")));
        }

        Ok(())
    }

    pub fn validate_username(username: &str) -> Result<(), ValidationError> {
        if username.is_empty() {
            return Err(ValidationError::EmptyField);
        }

        if username.len() < 3 {
            return Err(ValidationError::Custom(String::from("Username must be at least 3 characters")));
        }

        if username.len() > 20 {
            return Err(ValidationError::Custom(String::from("Username cannot exceed 20 characters")));
        }

        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidFormat);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(Validator::validate_email("test@example.com").is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Validator::validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_valid_age() {
        assert!(Validator::validate_age(25).is_ok());
    }

    #[test]
    fn test_invalid_age() {
        assert!(Validator::validate_age(15).is_err());
    }

    #[test]
    fn test_valid_username() {
        assert!(Validator::validate_username("user_123").is_ok());
    }

    #[test]
    fn test_invalid_username() {
        assert!(Validator::validate_username("ab").is_err());
    }
}