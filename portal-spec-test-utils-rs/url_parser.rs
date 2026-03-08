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
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.org:8080/page"),
            Some("sub.domain.org:8080".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("example.com/resource"),
            Some("example.com".to_string())
        );
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn extract_domain(&self) -> Option<String> {
        let re = Regex::new(r"https?://([^/]+)").unwrap();
        re.captures(&self.url)
            .map(|caps| caps[1].to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = self.url.find('?');
        
        if let Some(start) = query_start {
            let query_string = &self.url[start + 1..];
            
            for pair in query_string.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(
                        parts[0].to_string(),
                        parts[1].to_string()
                    );
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        re.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=value");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://example.com?name=john&age=30");
        let params = parser.parse_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid_parser = UrlParser::new("https://example.com");
        let invalid_parser = UrlParser::new("not-a-url");
        
        assert!(valid_parser.is_valid_url());
        assert!(!invalid_parser.is_valid_url());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let remaining = &url[start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain = &remaining[..domain_end];

        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                if let Some(equal_pos) = pair.find('=') {
                    let key = &pair[..equal_pos];
                    let value = &pair[equal_pos + 1..];
                    
                    if !key.is_empty() {
                        params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        params
    }

    pub fn extract_path(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let remaining = &url[start..];
        if let Some(slash_pos) = remaining.find('/') {
            let path = &remaining[slash_pos..];
            
            if let Some(query_pos) = path.find('?') {
                Some(path[..query_pos].to_string())
            } else {
                Some(path.to_string())
            }
        } else {
            Some("/".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(
            UrlParser::parse_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://sub.domain.co.uk:8080"),
            Some("sub.domain.co.uk:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid"), Some("invalid".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?key1=value1&key2=value2");
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"value2".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users?page=2"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
        assert_eq!(UrlParser::extract_path(""), None);
    }
}
use std::collections::HashMap;

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

pub fn get_query_param(query: &str, key: &str) -> Option<String> {
    parse_query_string(query).get(key).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_query() {
        let params = parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_single_param() {
        let params = parse_query_string("name=john");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_parse_multiple_params() {
        let params = parse_query_string("name=john&age=30&city=newyork");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
    }

    #[test]
    fn test_get_specific_param() {
        assert_eq!(get_query_param("name=john&age=30", "name"), Some("john".to_string()));
        assert_eq!(get_query_param("name=john&age=30", "age"), Some("30".to_string()));
        assert_eq!(get_query_param("name=john&age=30", "city"), None);
    }
}