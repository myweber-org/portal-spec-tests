
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyString,
    InvalidLength(usize, usize),
    InvalidFormat(String),
    OutOfRange(f64, f64, f64),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyString => write!(f, "String cannot be empty"),
            ValidationError::InvalidLength(min, max) => 
                write!(f, "Length must be between {} and {} characters", min, max),
            ValidationError::InvalidFormat(expected) => 
                write!(f, "Format must match: {}", expected),
            ValidationError::OutOfRange(value, min, max) => 
                write!(f, "Value {} is outside allowed range [{}, {}]", value, min, max),
        }
    }
}

impl Error for ValidationError {}

pub struct DataValidator;

impl DataValidator {
    pub fn validate_string_length(
        input: &str, 
        min_len: usize, 
        max_len: usize
    ) -> Result<(), ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::EmptyString);
        }
        
        let len = input.len();
        if len < min_len || len > max_len {
            return Err(ValidationError::InvalidLength(min_len, max_len));
        }
        
        Ok(())
    }
    
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        Self::validate_string_length(email, 3, 254)?;
        
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let regex = regex::Regex::new(email_pattern).unwrap();
        
        if !regex.is_match(email) {
            return Err(ValidationError::InvalidFormat("valid email address".to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_numeric_range(
        value: f64, 
        min: f64, 
        max: f64
    ) -> Result<(), ValidationError> {
        if value < min || value > max {
            return Err(ValidationError::OutOfRange(value, min, max));
        }
        
        Ok(())
    }
    
    pub fn validate_alphanumeric(input: &str) -> Result<(), ValidationError> {
        Self::validate_string_length(input, 1, 100)?;
        
        if !input.chars().all(|c| c.is_alphanumeric()) {
            return Err(ValidationError::InvalidFormat("alphanumeric characters only".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_string_length() {
        assert!(DataValidator::validate_string_length("test", 1, 10).is_ok());
        assert_eq!(
            DataValidator::validate_string_length("", 1, 10),
            Err(ValidationError::EmptyString)
        );
        assert_eq!(
            DataValidator::validate_string_length("a", 2, 10),
            Err(ValidationError::InvalidLength(2, 10))
        );
    }
    
    #[test]
    fn test_validate_email() {
        assert!(DataValidator::validate_email("user@example.com").is_ok());
        assert!(DataValidator::validate_email("invalid-email").is_err());
    }
    
    #[test]
    fn test_validate_numeric_range() {
        assert!(DataValidator::validate_numeric_range(5.0, 0.0, 10.0).is_ok());
        assert_eq!(
            DataValidator::validate_numeric_range(15.0, 0.0, 10.0),
            Err(ValidationError::OutOfRange(15.0, 0.0, 10.0))
        );
    }
    
    #[test]
    fn test_validate_alphanumeric() {
        assert!(DataValidator::validate_alphanumeric("abc123").is_ok());
        assert!(DataValidator::validate_alphanumeric("abc 123").is_err());
    }
}use regex::Regex;
use std::error::Error;

pub struct Validator {
    email_regex: Regex,
    phone_regex: Regex,
}

impl Validator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?,
            phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$")?,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_phone_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_phone("+1234567890"));
        assert!(!validator.validate_phone("abc"));
    }

    #[test]
    fn test_sanitization() {
        let validator = Validator::new().unwrap();
        let sanitized = validator.sanitize_input("Hello<script>alert('xss')</script>World!");
        assert_eq!(sanitized, "HelloalertxssWorld");
    }
}