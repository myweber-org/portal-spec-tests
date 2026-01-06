use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    params.insert(
                        key.to_string(),
                        urlencoding::decode(value)
                            .unwrap_or_else(|_| value.into())
                            .to_string()
                    );
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url = url.trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("www.");
        
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
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://subdomain.example.org/"),
            Some("subdomain.example.org".to_string())
        );
    }
}