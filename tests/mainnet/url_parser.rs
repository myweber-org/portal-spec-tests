
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_regex: Regex,
    tlds: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let domain_pattern = r"^(?:https?://)?(?:www\.)?([a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*\.[a-zA-Z]{2,})";
        let tld_list = vec![
            "com", "org", "net", "edu", "gov", "io", "co", "uk", "de", "fr",
            "jp", "cn", "ru", "br", "es", "ca", "au", "in", "mx", "it",
        ];

        UrlParser {
            domain_regex: Regex::new(domain_pattern).unwrap(),
            tlds: tld_list.into_iter().map(String::from).collect(),
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        self.domain_regex.captures(url).and_then(|caps| {
            caps.get(1).map(|m| m.as_str().to_lowercase())
        })
    }

    pub fn validate_tld(&self, domain: &str) -> bool {
        domain.split('.').last()
            .map(|tld| self.tlds.contains(tld))
            .unwrap_or(false)
    }

    pub fn parse(&self, url: &str) -> Option<ParsedUrl> {
        self.extract_domain(url).and_then(|domain| {
            if self.validate_tld(&domain) {
                Some(ParsedUrl {
                    original: url.to_string(),
                    domain,
                    is_secure: url.starts_with("https"),
                })
            } else {
                None
            }
        })
    }
}

pub struct ParsedUrl {
    pub original: String,
    pub domain: String,
    pub is_secure: bool,
}

impl ParsedUrl {
    pub fn display(&self) -> String {
        format!(
            "Domain: {} | Secure: {} | Original: {}",
            self.domain, self.is_secure, self.original
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let parser = UrlParser::new();
        
        let test_cases = vec![
            ("https://example.com", Some("example.com")),
            ("http://sub.domain.co.uk", Some("sub.domain.co.uk")),
            ("www.github.io", Some("github.io")),
            ("invalid.tld", None),
            ("https://test.xyz", None),
        ];

        for (input, expected) in test_cases {
            let result = parser.parse(input);
            assert_eq!(
                result.map(|r| r.domain),
                expected.map(String::from),
                "Failed for: {}",
                input
            );
        }
    }

    #[test]
    fn test_secure_detection() {
        let parser = UrlParser::new();
        
        let https_url = parser.parse("https://secure-site.com").unwrap();
        let http_url = parser.parse("http://insecure-site.com").unwrap();
        
        assert!(https_url.is_secure);
        assert!(!http_url.is_secure);
    }
}
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
        
        UrlParser {
            domain_blacklist: blacklist,
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        let pattern = r"^(?:https?://)?(?:www\.)?([^/:]+)";
        let re = Regex::new(pattern).unwrap();
        
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_lowercase())
            .filter(|domain| !self.domain_blacklist.contains(domain))
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        let url_pattern = r"^(https?://)?(www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(/.*)?$";
        let re = Regex::new(url_pattern).unwrap();
        
        re.is_match(url) && self.extract_domain(url).is_some()
    }

    pub fn normalize_url(&self, url: &str) -> Option<String> {
        if !self.is_valid_url(url) {
            return None;
        }

        let domain = self.extract_domain(url)?;
        let protocol = if url.starts_with("https") { "https" } else { "http" };
        
        Some(format!("{}://{}", protocol, domain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(parser.extract_domain("http://www.github.com/user/repo"), Some("github.com".to_string()));
        assert_eq!(parser.extract_domain("invalid-url"), None);
        assert_eq!(parser.extract_domain("http://localhost:8080"), None);
    }

    #[test]
    fn test_url_validation() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://github.com"));
        assert!(!parser.is_valid_url("not-a-url"));
        assert!(!parser.is_valid_url("http://localhost"));
    }

    #[test]
    fn test_url_normalization() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.normalize_url("example.com"), Some("http://example.com".to_string()));
        assert_eq!(parser.normalize_url("https://www.github.com"), Some("https://github.com".to_string()));
        assert_eq!(parser.normalize_url("invalid"), None);
    }
}