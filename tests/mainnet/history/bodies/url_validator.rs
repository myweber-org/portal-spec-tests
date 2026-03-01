use regex::Regex;
use std::error::Error;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let pattern = Regex::new(r"^https?://(?:www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?$")?;
        Ok(UrlValidator { pattern })
    }

    pub fn validate(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        self.pattern.captures(url).and_then(|caps| {
            caps.get(0).map(|m| {
                let full_url = m.as_str();
                let start = full_url.find("://").map(|i| i + 3).unwrap_or(0);
                let end = full_url[start..].find('/').map(|i| start + i).unwrap_or(full_url.len());
                full_url[start..end].to_string()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(validator.validate("https://example.com"));
        assert!(validator.validate("http://sub.example.com/path"));
        assert!(validator.validate("https://www.example.co.uk/resource"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(!validator.validate("example.com"));
        assert!(!validator.validate("ftp://example.com"));
        assert!(!validator.validate("https://example"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new().unwrap();
        assert_eq!(validator.extract_domain("https://api.github.com/users"), Some("api.github.com".to_string()));
        assert_eq!(validator.extract_domain("http://localhost:8080"), None);
    }
}