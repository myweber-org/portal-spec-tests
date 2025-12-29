
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub port: Option<u16>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        if url.trim().is_empty() {
            return Err("Empty URL provided".to_string());
        }

        let mut protocol = "http".to_string();
        let mut remaining = url;

        if let Some(proto_end) = url.find("://") {
            protocol = url[..proto_end].to_string().to_lowercase();
            remaining = &url[proto_end + 3..];
        }

        let mut domain_port = remaining;
        let mut path = "/".to_string();
        let mut query_params = HashMap::new();

        if let Some(path_start) = remaining.find('/') {
            domain_port = &remaining[..path_start];
            let path_and_query = &remaining[path_start..];

            if let Some(query_start) = path_and_query.find('?') {
                path = path_and_query[..query_start].to_string();
                let query_str = &path_and_query[query_start + 1..];

                for pair in query_str.split('&') {
                    if pair.is_empty() {
                        continue;
                    }
                    let mut parts = pair.split('=');
                    let key = parts.next().unwrap_or("").to_string();
                    let value = parts.next().unwrap_or("").to_string();
                    if !key.is_empty() {
                        query_params.insert(key, value);
                    }
                }
            } else {
                path = path_and_query.to_string();
            }
        }

        let mut domain = domain_port.to_string();
        let mut port = None;

        if let Some(port_start) = domain_port.find(':') {
            domain = domain_port[..port_start].to_string();
            if let Ok(parsed_port) = domain_port[port_start + 1..].parse::<u16>() {
                port = Some(parsed_port);
            }
        }

        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            domain,
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

    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}", self.protocol, self.domain);

        if let Some(port) = self.port {
            result.push_str(&format!(":{}", port));
        }

        result.push_str(&self.path);

        if !self.query_params.is_empty() {
            result.push('?');
            let query_parts: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            result.push_str(&query_parts.join("&"));
        }

        result
    }
}

pub fn is_valid_url(url: &str) -> bool {
    ParsedUrl::new(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url() {
        let url = ParsedUrl::new("https://example.com/path").unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, "example.com");
        assert_eq!(url.path, "/path");
        assert!(url.query_params.is_empty());
        assert_eq!(url.port, None);
    }

    #[test]
    fn test_url_with_query() {
        let url = ParsedUrl::new("http://example.com/search?q=rust&lang=en").unwrap();
        assert_eq!(url.protocol, "http");
        assert_eq!(url.domain, "example.com");
        assert_eq!(url.path, "/search");
        assert_eq!(url.query_params.len(), 2);
        assert_eq!(url.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(url.get_query_param("lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_url_with_port() {
        let url = ParsedUrl::new("https://localhost:8080/api").unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.domain, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
    }

    #[test]
    fn test_invalid_url() {
        assert!(ParsedUrl::new("").is_err());
        assert!(ParsedUrl::new("://example.com").is_err());
    }

    #[test]
    fn test_to_string() {
        let url = ParsedUrl::new("https://example.com:3000/search?q=test&page=2").unwrap();
        let reconstructed = url.to_string();
        assert!(reconstructed.contains("https://example.com:3000/search"));
        assert!(reconstructed.contains("q=test"));
        assert!(reconstructed.contains("page=2"));
    }
}