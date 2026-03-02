use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let pattern = r"^(https?|ftp)://[^\s/$.?#].[^\s]*$";
    let re = Regex::new(pattern).unwrap();
    re.is_match(url)
}

pub fn extract_domain(url: &str) -> Option<String> {
    if !is_valid_url(url) {
        return None;
    }
    
    let domain_pattern = r"^(?:https?://)?([^/:]+)";
    let re = Regex::new(domain_pattern).unwrap();
    
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://www.google.com/search?q=test"));
        assert!(is_valid_url("ftp://files.example.com/data.txt"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("example.com"));
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://www.rust-lang.org"), Some("www.rust-lang.org".to_string()));
        assert_eq!(extract_domain("http://localhost:8080"), Some("localhost".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }
}