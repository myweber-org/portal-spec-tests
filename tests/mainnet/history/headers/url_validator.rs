use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(r"^https?://(?:www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?$")
            .expect("Invalid regex pattern");
        UrlValidator { pattern }
    }

    pub fn validate(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.validate("https://example.com"));
        assert!(validator.validate("http://sub.example.com/path"));
        assert!(validator.validate("https://www.example.co.uk"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.validate("example.com"));
        assert!(!validator.validate("ftp://example.com"));
        assert!(!validator.validate("https://example"));
    }
}