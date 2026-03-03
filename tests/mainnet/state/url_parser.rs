use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
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
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://api.service.net/v1/endpoint";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("api.service.net".to_string()));
    }
}use std::collections::HashMap;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format");
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

        let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
        let path = if !path_query_split[0].is_empty() {
            format!("/{}", path_query_split[0])
        } else {
            "/".to_string()
        };

        let mut query_params = HashMap::new();
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
            domain,
            path,
            query_params,
        })
    }

    pub fn full_path(&self) -> String {
        if self.query_params.is_empty() {
            self.path.clone()
        } else {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("{}?{}", self.path, query_string.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_parse_root_domain() {
        let url = "http://localhost:8080";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "localhost:8080");
        assert_eq!(parsed.path, "/");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_full_path_with_query() {
        let url = "https://example.com/page?name=test&id=42";
        let parsed = ParsedUrl::parse(url).unwrap();
        let full_path = parsed.full_path();
        assert!(full_path.contains("name=test"));
        assert!(full_path.contains("id=42"));
        assert!(full_path.starts_with("/page?"));
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"https?://([^/]+)").unwrap();
        re.captures(url).map(|caps| caps[1].to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    params.insert(key.to_string(), value.to_string());
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
}use regex::Regex;

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<scheme>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    Some(ParsedUrl {
        scheme: captures.name("scheme")?.as_str().to_string(),
        host: captures.name("host")?.as_str().to_string(),
        path: captures
            .name("path")
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "/".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let parsed = parse_url("http://example.com/path/to/resource").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let parsed = parse_url("https://example.com").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let parsed = parse_url("not-a-valid-url");
        assert!(parsed.is_none());
    }
}use std::collections::HashMap;
use url::Url;

pub struct UrlParser {
    url: Url,
}

impl UrlParser {
    pub fn new(url_str: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(url_str)?;
        Ok(Self { url })
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.host_str()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn path_segments(&self) -> Vec<String> {
        self.url.path_segments()
            .map(|segments| segments.map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::new("https://example.com/api/v1/users?id=123&name=test").unwrap();
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.scheme(), "https");
        
        let params = parser.query_params();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("name"), Some(&"test".to_string()));
        
        let path = parser.path_segments();
        assert_eq!(path, vec!["api", "v1", "users"]);
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlParser::new("not-a-valid-url");
        assert!(result.is_err());
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^https?://([^/]+)").unwrap();
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
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    params.insert(key.to_string(), value.to_string());
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
        let url = "https://www.example.com/path?query=123";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let invalid = "not-a-url";
        assert_eq!(UrlParser::parse_domain(invalid), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30&city=nyc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"nyc".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://localhost:8080"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
        assert!(!UrlParser::is_valid_url("just-a-string"));
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    MalformedQuery,
    InvalidEncoding,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MalformedQuery => write!(f, "malformed query string"),
            ParseError::InvalidEncoding => write!(f, "invalid percent encoding"),
        }
    }
}

impl Error for ParseError {}

pub fn parse_query(query: &str) -> Result<HashMap<String, String>, ParseError> {
    if query.is_empty() {
        return Ok(HashMap::new());
    }

    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        
        let key = parts.next().ok_or(ParseError::MalformedQuery)?;
        let value = parts.next().unwrap_or("");
        
        let decoded_key = percent_decode(key).map_err(|_| ParseError::InvalidEncoding)?;
        let decoded_value = percent_decode(value).map_err(|_| ParseError::InvalidEncoding)?;
        
        params.insert(decoded_key, decoded_value);
    }
    
    Ok(params)
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let mut decoded = String::new();
    let mut chars = input.chars().collect::<Vec<_>>();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex_str: String = chars[i + 1..i + 3].iter().collect();
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                decoded.push(byte as char);
                i += 3;
                continue;
            }
        }
        decoded.push(chars[i]);
        i += 1;
    }
    
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let result = parse_query("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_param() {
        let result = parse_query("name=john").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_multiple_params() {
        let result = parse_query("name=john&age=25&city=new+york").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"25".to_string()));
        assert_eq!(result.get("city"), Some(&"new york".to_string()));
    }

    #[test]
    fn test_percent_encoding() {
        let result = parse_query("message=hello%20world%21").unwrap();
        assert_eq!(result.get("message"), Some(&"hello world!".to_string()));
    }

    #[test]
    fn test_malformed_query() {
        let result = parse_query("key=&=value");
        assert!(matches!(result, Err(ParseError::MalformedQuery)));
    }
}