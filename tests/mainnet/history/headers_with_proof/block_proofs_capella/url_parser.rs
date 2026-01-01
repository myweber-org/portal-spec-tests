use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"https?://([^/]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=test";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let invalid = "not-a-url";
        assert_eq!(UrlParser::parse_domain(invalid), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=25&city=ny";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
        assert_eq!(params.get("city"), Some(&"ny".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://localhost:8080"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
        assert!(!UrlParser::is_valid_url("example.com"));
    }
}
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for param_pair in query_string.split('&') {
                let parts: Vec<&str> = param_pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            if let Some(domain_start) = url.find("://") {
                let after_protocol = &url[domain_start + 3..];
                if let Some(domain_end) = after_protocol.find('/') {
                    return Some(after_protocol[..domain_end].to_string());
                }
                return Some(after_protocol.to_string());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=1";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://subdomain.example.co.uk";
        let url3 = "invalid-url";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("subdomain.example.co.uk".to_string()));
        assert_eq!(UrlParser::extract_domain(url3), None);
    }
    
    #[test]
    fn test_empty_query_params() {
        let url = "https://example.com";
        let params = UrlParser::parse_query_params(url);
        assert!(params.is_empty());
    }
}