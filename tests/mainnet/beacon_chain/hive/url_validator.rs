use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"^https?://([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(:\d+)?(/[^\s]*)?$"
        ).expect("Invalid regex pattern");
        
        UrlValidator { pattern }
    }

    pub fn is_valid_url(&self, input: &str) -> bool {
        self.pattern.is_match(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.is_valid_url("https://example.com"));
        assert!(validator.is_valid_url("http://sub.domain.co.uk/path"));
        assert!(validator.is_valid_url("https://api.service.io:8080/endpoint"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.is_valid_url("not-a-url"));
        assert!(!validator.is_valid_url("ftp://invalid.protocol"));
        assert!(!validator.is_valid_url("http://"));
    }
}