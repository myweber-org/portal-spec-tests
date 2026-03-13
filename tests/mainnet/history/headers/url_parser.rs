
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
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        let scheme = parts[0].to_string();
        let rest = parts[1];

        let host_end = rest.find('/').unwrap_or(rest.len());
        let host = rest[..host_end].to_string();
        let path_and_query = &rest[host_end..];

        let path_parts: Vec<&str> = path_and_query.split('?').collect();
        let path = path_parts[0].to_string();
        let mut query_params = HashMap::new();

        if path_parts.len() > 1 {
            for pair in path_parts[1].split('&') {
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            scheme,
            host,
            path,
            query_params,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
        assert!(!parsed.has_query_params());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.service.com/data?page=2&limit=50";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "api.service.com");
        assert_eq!(parsed.path, "/data");
        assert_eq!(parsed.get_query_param("page"), Some(&"2".to_string()));
        assert_eq!(parsed.get_query_param("limit"), Some(&"50".to_string()));
        assert_eq!(parsed.get_query_param("missing"), None);
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}