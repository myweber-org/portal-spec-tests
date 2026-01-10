use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let after_protocol = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            if let Some(slash_pos) = after_protocol.find('/') {
                return Some(after_protocol[..slash_pos].to_string());
            }
            return Some(after_protocol.to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://api.service.net/v1/endpoint";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("api.service.net".to_string()));
    }
}