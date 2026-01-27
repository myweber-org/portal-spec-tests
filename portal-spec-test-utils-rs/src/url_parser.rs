
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") {
            url_lower.strip_prefix("http://")
                .and_then(|s| s.split('/').next())
                .map(|s| s.to_string())
        } else if url_lower.starts_with("https://") {
            url_lower.strip_prefix("https://")
                .and_then(|s| s.split('/').next())
                .map(|s| s.to_string())
        } else {
            url.split('/').next().map(|s| s.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain_https() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("www.example.com".to_string()));
    }

    #[test]
    fn test_extract_domain_http() {
        let url = "http://example.com:8080/api";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("example.com:8080".to_string()));
    }

    #[test]
    fn test_extract_domain_no_protocol() {
        let url = "example.com/page";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("example.com".to_string()));
    }
}