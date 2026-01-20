
use regex::Regex;
use std::collections::HashSet;

pub struct Validator {
    email_regex: Regex,
    forbidden_usernames: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let mut forbidden = HashSet::new();
        forbidden.insert("admin".to_string());
        forbidden.insert("root".to_string());
        forbidden.insert("system".to_string());

        Validator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_usernames: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email.trim())
    }

    pub fn validate_username(&self, username: &str) -> Result<(), String> {
        let name = username.trim();
        
        if name.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }
        
        if name.len() > 20 {
            return Err("Username must not exceed 20 characters".to_string());
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain alphanumeric characters, underscores and hyphens".to_string());
        }
        
        if self.forbidden_usernames.contains(&name.to_lowercase()) {
            return Err("This username is not allowed".to_string());
        }
        
        Ok(())
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));
        
        password.len() >= 8 && has_upper && has_lower && has_digit && has_special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(validator.validate_email("test.user+tag@domain.co.uk"));
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("user@.com"));
    }

    #[test]
    fn test_username_validation() {
        let validator = Validator::new();
        assert!(validator.validate_username("valid_user").is_ok());
        assert!(validator.validate_username("user-123").is_ok());
        assert!(validator.validate_username("ab").is_err());
        assert!(validator.validate_username("admin").is_err());
        assert!(validator.validate_username("username_with_invalid!char").is_err());
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongP@ss1"));
        assert!(!validator.validate_password_strength("weak"));
        assert!(!validator.validate_password_strength("NoSpecial1"));
        assert!(!validator.validate_password_strength("noupper1@"));
    }
}