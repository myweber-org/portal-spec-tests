use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() < 2 {
            return None;
        }

        let domain_part = parts[1];
        let domain_end = domain_part.find('/').unwrap_or(domain_part.len());
        let domain = &domain_part[..domain_end];

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
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    params.insert(
                        kv[0].to_string(),
                        kv[1].to_string()
                    );
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

        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };

        if let Some(path_start) = after_protocol.find('/') {
            let path = &after_protocol[path_start..];
            
            if let Some(query_start) = path.find('?') {
                Some(path[..query_start].to_string())
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
            UrlParser::parse_domain("ftp://files.server.org"),
            Some("files.server.org".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/users"),
            Some("/api/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/search?q=test"),
            Some("/search".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        re.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=value";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let url_no_protocol = "example.com/page";
        assert_eq!(UrlParser::parse_domain(url_no_protocol), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30&city=nyc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"nyc".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("http://example.com"));
        assert!(UrlParser::is_valid_url("https://sub.example.com/path"));
        assert!(!UrlParser::is_valid_url("not-a-url"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let parts: Vec<&str> = url.splitn(2, "://").collect();
    if parts.len() != 2 {
        return None;
    }
    let scheme = parts[0].to_string();
    let rest = parts[1];

    let mut host_path_split = rest.splitn(2, '/');
    let host = host_path_split.next().unwrap_or("").to_string();
    let path_with_query = host_path_split.next().unwrap_or("").to_string();

    let mut path_query_split = path_with_query.splitn(2, '?');
    let path = path_query_split.next().unwrap_or("").to_string();
    let query = path_query_split.next().unwrap_or("");

    let mut query_params = HashMap::new();
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            if !key.is_empty() {
                query_params.insert(key.to_string(), value.to_string());
            }
        }
    }

    Some(ParsedUrl {
        scheme,
        host,
        path: if path.is_empty() { "/".to_string() } else { format!("/{}", path) },
        query_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://www.example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "www.example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "http://localhost:8080/api/data?key=value&sort=asc";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "localhost:8080");
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"asc".to_string()));
    }

    #[test]
    fn test_parse_url_no_path() {
        let url = "ftp://fileserver.net";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "ftp");
        assert_eq!(parsed.host, "fileserver.net");
        assert_eq!(parsed.path, "/");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}