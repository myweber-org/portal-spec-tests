
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_blacklist: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let mut blacklist = HashSet::new();
        blacklist.insert("localhost".to_string());
        blacklist.insert("127.0.0.1".to_string());
        blacklist.insert("::1".to_string());
        blacklist.insert("0.0.0.0".to_string());
        
        UrlParser {
            domain_blacklist: blacklist,
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?(?:www\.)?([^:/]+)").unwrap();
        
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_lowercase())
            .filter(|domain| !self.domain_blacklist.contains(domain))
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        let url_pattern = Regex::new(
            r"^(https?://)?(www\.)?[a-zA-Z0-9-]+(\.[a-zA-Z]{2,})+(:\d+)?(/[^\s]*)?$"
        ).unwrap();
        
        url_pattern.is_match(url) && self.extract_domain(url).is_some()
    }

    pub fn normalize_url(&self, url: &str) -> Option<String> {
        if !self.is_valid_url(url) {
            return None;
        }

        let domain = self.extract_domain(url)?;
        let re = Regex::new(r"^(https?://)?(www\.)?").unwrap();
        let clean_url = re.replace(url, "");
        
        Some(format!("https://www.{}", clean_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("http://subdomain.example.co.uk:8080"),
            Some("subdomain.example.co.uk".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("ftp://invalid.protocol"),
            None
        );
    }

    #[test]
    fn test_blacklist_validation() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.extract_domain("http://localhost/api"), None);
        assert_eq!(parser.extract_domain("https://127.0.0.1:3000"), None);
        assert_eq!(parser.is_valid_url("http://0.0.0.0"), false);
    }

    #[test]
    fn test_url_normalization() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.normalize_url("example.com"),
            Some("https://www.example.com".to_string())
        );
        
        assert_eq!(
            parser.normalize_url("http://example.com"),
            Some("https://www.example.com".to_string())
        );
        
        assert_eq!(
            parser.normalize_url("https://example.com/path?query=1"),
            Some("https://www.example.com/path?query=1".to_string())
        );
    }
}