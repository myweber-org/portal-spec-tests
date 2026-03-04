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
        let url = "https://www.example.com/path?query=value";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30";
        let params = UrlParser::parse_query_params(url);
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://example.com"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return None;
    }

    let protocol = parts[0].to_string();
    let rest = parts[1];

    let domain_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let domain = domain_path_split[0].to_string();
    
    let path_and_query = if domain_path_split.len() > 1 {
        domain_path_split[1]
    } else {
        ""
    };

    let path_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    let path = if !path_split[0].is_empty() {
        format!("/{}", path_split[0])
    } else {
        "/".to_string()
    };

    let mut query_params = HashMap::new();
    if path_split.len() > 1 {
        for pair in path_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Some(ParsedUrl {
        protocol,
        domain,
        path,
        query_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://example.com/search?q=rust&lang=en";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_parse_url_no_path() {
        let url = "ftp://fileserver.net";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.domain, "fileserver.net");
        assert_eq!(parsed.path, "/");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}