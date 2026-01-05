use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"
        ).expect("Invalid regex pattern");
        
        UrlValidator { pattern }
    }

    pub fn is_valid(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.is_valid(url) {
            return None;
        }
        
        let domain_pattern = Regex::new(r"https?://(?:www\.)?([^/]+)").unwrap();
        domain_pattern.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.is_valid("https://example.com"));
        assert!(validator.is_valid("http://sub.example.co.uk/path?query=value"));
        assert!(validator.is_valid("https://www.google.com/search?q=rust"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.is_valid("not-a-url"));
        assert!(!validator.is_valid("ftp://example.com"));
        assert!(!validator.is_valid("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new();
        assert_eq!(
            validator.extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        assert_eq!(
            validator.extract_domain("http://www.example.co.uk/path"),
            Some("www.example.co.uk".to_string())
        );
        assert_eq!(validator.extract_domain("invalid-url"), None);
    }
}