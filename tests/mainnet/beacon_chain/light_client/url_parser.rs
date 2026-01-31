
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let mut chars = url.chars().peekable();
        
        while let Some(&c) = chars.peek() {
            match c {
                ':' if chars.clone().take(3).collect::<String>() == "://" => {
                    chars.next(); chars.next(); chars.next();
                    break;
                }
                _ => {
                    protocol.push(c);
                    chars.next();
                }
            }
        }

        while let Some(&c) = chars.peek() {
            match c {
                '/' | '?' | '#' => break,
                ':' => {
                    chars.next();
                    let port_str: String = chars.by_ref()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !port_str.is_empty() {
                        port = Some(port_str.parse().map_err(|_| "Invalid port")?);
                    }
                    break;
                }
                _ => {
                    host.push(c);
                    chars.next();
                }
            }
        }

        if let Some('/') = chars.peek() {
            chars.next();
            while let Some(&c) = chars.peek() {
                match c {
                    '?' | '#' => break,
                    _ => {
                        path.push(c);
                        chars.next();
                    }
                }
            }
        }

        if let Some('?') = chars.peek() {
            chars.next();
            let query_str: String = chars.by_ref()
                .take_while(|&c| c != '#')
                .collect();
            
            for pair in query_str.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    if !key.is_empty() {
                        query_params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        if let Some('#') = chars.peek() {
            chars.next();
            let frag: String = chars.collect();
            if !frag.is_empty() {
                fragment = Some(frag);
            }
        }

        if host.is_empty() {
            return Err("Missing host".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            host,
            port,
            path: if path.is_empty() { "/".to_string() } else { path },
            query_params,
            fragment,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }

    pub fn build_url(&self) -> String {
        let mut result = format!("{}://{}", self.protocol, self.host);
        
        if let Some(port) = self.port {
            result.push_str(&format!(":{}", port));
        }
        
        result.push_str(&self.path);
        
        if self.has_query_params() {
            result.push('?');
            let params: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            result.push_str(&params.join("&"));
        }
        
        if let Some(fragment) = &self.fragment {
            result.push_str(&format!("#{}", fragment));
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url() {
        let url = ParsedUrl::new("https://example.com/path").unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");
        assert!(!url.has_query_params());
        assert_eq!(url.fragment, None);
    }

    #[test]
    fn test_url_with_query() {
        let url = ParsedUrl::new("http://localhost:8080/api?search=rust&page=1").unwrap();
        assert_eq!(url.protocol, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
        assert_eq!(url.get_query_param("search"), Some(&"rust".to_string()));
        assert_eq!(url.get_query_param("page"), Some(&"1".to_string()));
        assert_eq!(url.get_query_param("missing"), None);
    }

    #[test]
    fn test_url_reconstruction() {
        let original = "https://api.service.com/v1/data?filter=active&sort=desc#section";
        let parsed = ParsedUrl::new(original).unwrap();
        let reconstructed = parsed.build_url();
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_invalid_url() {
        let result = ParsedUrl::new("://missing-host");
        assert!(result.is_err());
    }
}