use regex::Regex;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidFormat,
    UnsupportedProtocol,
    MissingHost,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidFormat => write!(f, "URL format is invalid"),
            ValidationError::UnsupportedProtocol => write!(f, "URL protocol must be http or https"),
            ValidationError::MissingHost => write!(f, "URL must contain a hostname"),
        }
    }
}

impl Error for ValidationError {}

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(
            r"^(https?)://([a-zA-Z0-9\-\.]+)(?::(\d+))?(?:/(.*))?$"
        )?;
        
        Ok(UrlValidator { pattern })
    }

    pub fn validate(&self, url: &str) -> Result<(), ValidationError> {
        let captures = match self.pattern.captures(url) {
            Some(caps) => caps,
            None => return Err(ValidationError::InvalidFormat),
        };

        let protocol = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        if protocol != "http" && protocol != "https" {
            return Err(ValidationError::UnsupportedProtocol);
        }

        let host = captures.get(2).map(|m| m.as_str()).unwrap_or("");
        if host.is_empty() {
            return Err(ValidationError::MissingHost);
        }

        Ok(())
    }

    pub fn extract_components(&self, url: &str) -> Option<(String, String, Option<u16>, String)> {
        self.pattern.captures(url).map(|caps| {
            let protocol = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let host = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let port = caps.get(3).and_then(|m| m.as_str().parse::<u16>().ok());
            let path = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();
            
            (protocol, host, port, path)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://localhost:8080/api").is_ok());
        assert!(validator.validate("https://sub.domain.co.uk/path/to/resource").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        
        assert_eq!(validator.validate("ftp://example.com"), Err(ValidationError::UnsupportedProtocol));
        assert_eq!(validator.validate("https://"), Err(ValidationError::MissingHost));
        assert_eq!(validator.validate("invalid-url"), Err(ValidationError::InvalidFormat));
    }

    #[test]
    fn test_component_extraction() {
        let validator = UrlValidator::new().unwrap();
        
        let components = validator.extract_components("https://example.com:443/api/v1");
        assert_eq!(components, Some(("https".to_string(), "example.com".to_string(), Some(443), "api/v1".to_string())));
    }
}