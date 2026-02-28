use regex::Regex;
use std::error::Error;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let pattern = Regex::new(r"^https?://(?:[-\w.]|(?:%[\da-fA-F]{2}))+(?:/[-\w._~:/?#[\]@!$&'()*+,;=]*)?$")?;
        Ok(UrlValidator { pattern })
    }

    pub fn validate(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.validate(url) {
            return None;
        }
        
        let domain_start = url.find("://").map(|pos| pos + 3).unwrap_or(0);
        let domain_end = url[domain_start..]
            .find(|c| c == '/' || c == '?' || c == '#')
            .map(|pos| domain_start + pos)
            .unwrap_or(url.len());
        
        Some(url[domain_start..domain_end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(validator.validate("https://example.com"));
        assert!(validator.validate("http://sub.domain.co.uk/path"));
        assert!(validator.validate("https://api.service.io?query=value"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(!validator.validate("not-a-url"));
        assert!(!validator.validate("ftp://invalid.protocol"));
        assert!(!validator.validate("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new().unwrap();
        assert_eq!(
            validator.extract_domain("https://api.github.com/users"),
            Some("api.github.com".to_string())
        );
        assert_eq!(
            validator.extract_domain("http://localhost:8080"),
            Some("localhost:8080".to_string())
        );
    }
}