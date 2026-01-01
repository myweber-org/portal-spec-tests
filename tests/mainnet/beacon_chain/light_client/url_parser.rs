use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl, String> {
    let mut protocol = String::new();
    let mut host = String::new();
    let mut port = None;
    let mut path = String::new();
    let mut query_params = HashMap::new();

    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid URL format".to_string());
    }

    protocol = parts[0].to_string();
    let rest = parts[1];

    let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let authority = host_path_split[0];
    let path_and_query = if host_path_split.len() > 1 {
        format!("/{}", host_path_split[1])
    } else {
        "/".to_string()
    };

    let host_port_split: Vec<&str> = authority.split(':').collect();
    host = host_port_split[0].to_string();

    if host_port_split.len() > 1 {
        if let Ok(p) = host_port_split[1].parse::<u16>() {
            port = Some(p);
        } else {
            return Err("Invalid port number".to_string());
        }
    }

    let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    path = path_query_split[0].to_string();

    if path_query_split.len() > 1 {
        for pair in path_query_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Ok(ParsedUrl {
        protocol,
        host,
        port,
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
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?page=1&limit=20";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"20".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "invalid-url";
        let result = parse_url(url);
        assert!(result.is_err());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
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
        let url = "https://example.com/search?q=rust&lang=en&sort=date";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"date".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_domain_extraction() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://subdomain.example.org:8080/api";
        let url3 = "invalid-url";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("subdomain.example.org:8080".to_string()));
        assert_eq!(UrlParser::extract_domain(url3), None);
    }
    
    #[test]
    fn test_empty_query() {
        let url = "https://example.com/page";
        let params = UrlParser::parse_query_params(url);
        assert!(params.is_empty());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<&str> {
        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };
        
        after_protocol.split('/').next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        assert_eq!(UrlParser::extract_domain("https://github.com/rust-lang"), Some("github.com"));
        assert_eq!(UrlParser::extract_domain("http://localhost:8080/api"), Some("localhost:8080"));
        assert_eq!(UrlParser::extract_domain("example.com/path"), Some("example.com"));
    }
}