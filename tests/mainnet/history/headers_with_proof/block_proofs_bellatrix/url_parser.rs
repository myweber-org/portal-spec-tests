
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub port: Option<u16>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut scheme = String::new();
        let mut host = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut port = None;

        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_query[0];
        let path_query = if host_path_query.len() > 1 {
            format!("/{}", host_path_query[1])
        } else {
            "/".to_string()
        };

        let host_port: Vec<&str> = authority.split(':').collect();
        host = host_port[0].to_string();
        if host_port.len() == 2 {
            port = Some(host_port[1].parse().map_err(|_| "Invalid port number")?);
        }

        let path_query_parts: Vec<&str> = path_query.splitn(2, '?').collect();
        path = path_query_parts[0].to_string();

        if path_query_parts.len() == 2 {
            let query_str = path_query_parts[1];
            for pair in query_str.split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
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
            port,
        })
    }

    pub fn has_query_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn is_secure(&self) -> bool {
        self.scheme == "https"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = ParsedUrl::parse("https://example.com/path").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.path, "/path");
        assert!(url.query_params.is_empty());
        assert_eq!(url.port, None);
        assert!(url.is_secure());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = ParsedUrl::parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
        assert!(!url.is_secure());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = ParsedUrl::parse("https://api.example.com/search?q=rust&limit=10").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "api.example.com");
        assert_eq!(url.path, "/search");
        assert_eq!(url.query_params.len(), 2);
        assert_eq!(url.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(url.get_query_param("limit"), Some(&"10".to_string()));
        assert!(url.has_query_param("q"));
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = ParsedUrl::parse("invalid-url");
        assert!(result.is_err());
    }
}