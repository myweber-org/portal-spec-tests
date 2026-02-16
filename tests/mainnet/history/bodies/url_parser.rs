use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        protocol = parts[0].to_string();
        let rest = parts[1];

        let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_query[0];
        let mut path_and_query = if host_path_query.len() > 1 {
            format!("/{}", host_path_query[1])
        } else {
            "/".to_string()
        };

        let host_port: Vec<&str> = authority.split(':').collect();
        host = host_port[0].to_string();
        if host_port.len() == 2 {
            port = Some(host_port[1].parse().map_err(|_| "Invalid port number")?);
        }

        if let Some(query_start) = path_and_query.find('?') {
            let query_str = &path_and_query[query_start + 1..];
            path = path_and_query[..query_start].to_string();

            for pair in query_str.split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        } else {
            path = path_and_query;
        }

        Ok(ParsedUrl {
            protocol,
            host,
            port,
            path,
            query_params,
        })
    }

    pub fn build_url(&self) -> String {
        let mut url = format!("{}://{}", self.protocol, self.host);
        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }
        url.push_str(&self.path);

        if !self.query_params.is_empty() {
            url.push('?');
            let query_parts: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&query_parts.join("&"));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = ParsedUrl::parse("https://example.com/path/to/resource").unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path/to/resource");
        assert!(url.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = ParsedUrl::parse("http://localhost:8080/api/data?page=1&limit=10").unwrap();
        assert_eq!(url.protocol, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api/data");
        assert_eq!(url.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(url.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_build_url() {
        let mut query_params = HashMap::new();
        query_params.insert("search".to_string(), "rust".to_string());
        query_params.insert("sort".to_string(), "desc".to_string());

        let parsed_url = ParsedUrl {
            protocol: "https".to_string(),
            host: "api.example.com".to_string(),
            port: Some(443),
            path: "/v1/users".to_string(),
            query_params,
        };

        let built = parsed_url.build_url();
        assert!(built.starts_with("https://api.example.com:443/v1/users?"));
        assert!(built.contains("search=rust"));
        assert!(built.contains("sort=desc"));
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

        let url_from_start = &url[start..];
        let domain_end = url_from_start.find('/').unwrap_or(url_from_start.len());
        let domain = &url_from_start[..domain_end];

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
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        let url = url.trim();
        if url.is_empty() {
            return false;
        }

        let url_lower = url.to_lowercase();
        let valid_schemes = ["http://", "https://", "ftp://"];
        
        for scheme in valid_schemes.iter() {
            if url_lower.starts_with(scheme) {
                let after_scheme = &url[scheme.len()..];
                return !after_scheme.is_empty() && after_scheme.contains('.');
            }
        }
        
        false
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
            UrlParser::parse_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=1";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.get("nonexistent"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://sub.domain.org/path"));
        assert!(!UrlParser::is_valid_url("example.com"));
        assert!(!UrlParser::is_valid_url(""));
        assert!(!UrlParser::is_valid_url("https://"));
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                params.insert(key, value);
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                return Some(after_protocol[..end].to_string());
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
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("www.example.com".to_string()));
    }

    #[test]
    fn test_extract_domain_no_path() {
        let url = "https://api.service.net";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("api.service.net".to_string()));
    }
}