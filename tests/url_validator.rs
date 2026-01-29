use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let regex_pattern = r"^(https?|ftp)://[^\s/$.?#].[^\s]*$";
        let compiled_regex = Regex::new(regex_pattern).expect("Invalid regex pattern");
        
        UrlValidator {
            pattern: compiled_regex
        }
    }

    pub fn is_valid_url(&self, input: &str) -> bool {
        self.pattern.is_match(input)
    }

    pub fn extract_urls(&self, text: &str) -> Vec<String> {
        self.pattern
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        
        assert!(validator.is_valid_url("http://example.com"));
        assert!(validator.is_valid_url("https://sub.domain.co.uk/path"));
        assert!(validator.is_valid_url("ftp://files.server.org/file.txt"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        
        assert!(!validator.is_valid_url("not-a-url"));
        assert!(!validator.is_valid_url("http://"));
        assert!(!validator.is_valid_url("example.com"));
    }

    #[test]
    fn test_url_extraction() {
        let validator = UrlValidator::new();
        let text = "Visit http://site.com and https://secure.site.org for info";
        let urls = validator.extract_urls(text);
        
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://site.com");
        assert_eq!(urls[1], "https://secure.site.org");
    }
}