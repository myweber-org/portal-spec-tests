
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
        let url = url.trim_start_matches("http://")
            .trim_start_matches("https://");
        
        if let Some(end) = url.find('/') {
            Some(url[..end].to_string())
        } else {
            Some(url.to_string())
        }
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
    }
    
    #[test]
    fn test_extract_domain() {
        let url1 = "https://example.com/path/to/resource";
        let url2 = "http://subdomain.example.com";
        let url3 = "example.com";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("subdomain.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url3), Some("example.com".to_string()));
    }
}