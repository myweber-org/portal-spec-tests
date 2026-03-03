
use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string()
                );
            }
        }
        
        params
    }
    
    pub fn get_param(query: &str, key: &str) -> Option<String> {
        Self::parse(query).get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query() {
        let query = "name=John%20Doe&age=30&city=New%20York";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
        assert_eq!(params.get("country"), None);
    }
    
    #[test]
    fn test_empty_query() {
        let params = QueryParser::parse("");
        assert!(params.is_empty());
    }
    
    #[test]
    fn test_get_specific_param() {
        let query = "token=abc123&expires=3600";
        let token = QueryParser::get_param(query, "token");
        assert_eq!(token, Some("abc123".to_string()));
        
        let missing = QueryParser::get_param(query, "missing");
        assert_eq!(missing, None);
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut scheme = String::new();
        let mut remaining = url_str;

        if let Some(pos) = url_str.find("://") {
            scheme = url_str[..pos].to_string();
            remaining = &url_str[pos + 3..];
        } else {
            return Err("Missing scheme separator".to_string());
        }

        let mut host_port = remaining;
        let mut path_query = "";

        if let Some(pos) = remaining.find('/') {
            host_port = &remaining[..pos];
            path_query = &remaining[pos..];
        } else if let Some(pos) = remaining.find('?') {
            host_port = &remaining[..pos];
            path_query = &remaining[pos..];
        }

        let mut host = host_port.to_string();
        let mut port = None;

        if let Some(pos) = host_port.find(':') {
            host = host_port[..pos].to_string();
            if let Ok(parsed_port) = host_port[pos + 1..].parse::<u16>() {
                port = Some(parsed_port);
            } else {
                return Err("Invalid port number".to_string());
            }
        }

        let mut path = String::new();
        let mut query_params = HashMap::new();

        if let Some(pos) = path_query.find('?') {
            path = path_query[..pos].to_string();
            let query_str = &path_query[pos + 1..];
            
            for pair in query_str.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = &pair[..eq_pos];
                    let value = &pair[eq_pos + 1..];
                    query_params.insert(key.to_string(), value.to_string());
                }
            }
        } else {
            path = path_query.to_string();
        }

        if path.is_empty() {
            path = "/".to_string();
        }

        Ok(ParsedUrl {
            scheme,
            host,
            port,
            path,
            query_params,
        })
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            result.push_str(&format!(":{}", port));
        }
        
        result.push_str(&self.path);
        
        if !self.query_params.is_empty() {
            result.push('?');
            let query_parts: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            result.push_str(&query_parts.join("&"));
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = ParsedUrl::parse("https://example.com/path/to/resource").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path/to/resource");
        assert!(url.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = ParsedUrl::parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = ParsedUrl::parse("https://api.example.com/search?q=rust&limit=10").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "api.example.com");
        assert_eq!(url.path, "/search");
        assert_eq!(url.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(url.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_round_trip() {
        let original = "https://example.com:3000/path?key=value&another=param";
        let parsed = ParsedUrl::parse(original).unwrap();
        let reconstructed = parsed.to_string();
        assert_eq!(original, reconstructed);
    }
}