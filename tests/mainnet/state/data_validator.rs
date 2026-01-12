use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ValidationError {
    RequiredFieldMissing(String),
    InvalidFormat(String),
    OutOfRange(String, f64, f64),
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::RequiredFieldMissing(field) => write!(f, "Required field '{}' is missing", field),
            ValidationError::InvalidFormat(field) => write!(f, "Field '{}' has invalid format", field),
            ValidationError::OutOfRange(field, min, max) => write!(f, "Field '{}' must be between {} and {}", field, min, max),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct Validator;

impl Validator {
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if email.trim().is_empty() {
            return Err(ValidationError::RequiredFieldMissing("email".to_string()));
        }
        
        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidFormat("email".to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_age(age: u8) -> Result<(), ValidationError> {
        const MIN_AGE: u8 = 18;
        const MAX_AGE: u8 = 120;
        
        if age < MIN_AGE || age > MAX_AGE {
            return Err(ValidationError::OutOfRange(
                "age".to_string(),
                MIN_AGE as f64,
                MAX_AGE as f64
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_not_empty(field_name: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::RequiredFieldMissing(field_name.to_string()));
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
    fn test_invalid_email_format() {
        assert!(Validator::validate_email("invalid-email").is_err());
    }
    
    #[test]
    fn test_valid_age() {
        assert!(Validator::validate_age(25).is_ok());
    }
    
    #[test]
    fn test_invalid_age_too_young() {
        assert!(Validator::validate_age(16).is_err());
    }
    
    #[test]
    fn test_validate_not_empty() {
        assert!(Validator::validate_not_empty("name", "John").is_ok());
        assert!(Validator::validate_not_empty("name", "").is_err());
    }
}