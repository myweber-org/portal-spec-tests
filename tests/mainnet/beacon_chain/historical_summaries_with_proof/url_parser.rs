use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/:]+)").unwrap();
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
        let url = "https://example.com/path?query=test";
        assert_eq!(UrlParser::parse_domain(url), Some("example.com".to_string()));
        
        let url_no_proto = "example.com/path";
        assert_eq!(UrlParser::parse_domain(url_no_proto), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=25&city=nyc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
        assert_eq!(params.get("city"), Some(&"nyc".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://example.com"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
        assert!(!UrlParser::is_valid_url("example.com"));
    }
}use std::collections::HashMap;

pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn from_url(url: &str) -> Result<Self, &'static str> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                if pair.is_empty() {
                    continue;
                }
                
                let mut parts = pair.splitn(2, '=');
                let key = parts.next().unwrap();
                let value = parts.next().unwrap_or("");
                
                let decoded_key = url_decode(key)?;
                let decoded_value = url_decode(value)?;
                
                params
                    .entry(decoded_key)
                    .or_insert_with(Vec::new)
                    .push(decoded_value);
            }
        }
        
        Ok(QueryParams { params })
    }
    
    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.params.get(key)
    }
    
    pub fn get_first(&self, key: &str) -> Option<&String> {
        self.params.get(key).and_then(|v| v.first())
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }
    
    pub fn len(&self) -> usize {
        self.params.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

fn url_decode(input: &str) -> Result<String, &'static str> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex1 = chars.next().ok_or("Invalid percent encoding")?;
            let hex2 = chars.next().ok_or("Invalid percent encoding")?;
            
            let byte = u8::from_str_radix(&format!("{}{}", hex1, hex2), 16)
                .map_err(|_| "Invalid hex digit in percent encoding")?;
            
            result.push(byte as char);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_query() {
        let params = QueryParams::from_url("https://example.com").unwrap();
        assert!(params.is_empty());
    }
    
    #[test]
    fn test_single_param() {
        let params = QueryParams::from_url("https://example.com?name=john").unwrap();
        assert_eq!(params.get_first("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_multiple_params() {
        let params = QueryParams::from_url("https://example.com?name=john&age=30").unwrap();
        assert_eq!(params.get_first("name"), Some(&"john".to_string()));
        assert_eq!(params.get_first("age"), Some(&"30".to_string()));
    }
    
    #[test]
    fn test_url_decoding() {
        let params = QueryParams::from_url("https://example.com?search=hello%20world").unwrap();
        assert_eq!(params.get_first("search"), Some(&"hello world".to_string()));
    }
    
    #[test]
    fn test_plus_decoding() {
        let params = QueryParams::from_url("https://example.com?q=rust+language").unwrap();
        assert_eq!(params.get_first("q"), Some(&"rust language".to_string()));
    }
    
    #[test]
    fn test_multiple_values() {
        let params = QueryParams::from_url("https://example.com?color=red&color=blue").unwrap();
        let colors = params.get("color").unwrap();
        assert_eq!(colors, &vec!["red".to_string(), "blue".to_string()]);
    }
    
    #[test]
    fn test_invalid_percent_encoding() {
        let result = QueryParams::from_url("https://example.com?invalid=%GG");
        assert!(result.is_err());
    }
}
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut rest = url;
        
        if let Some(proto_end) = url.find("://") {
            protocol = url[..proto_end].to_string();
            rest = &url[proto_end + 3..];
        }
        
        let mut domain_end = rest.find('/').unwrap_or(rest.len());
        let fragment_pos = rest.find('#');
        
        if let Some(frag_pos) = fragment_pos {
            if frag_pos < domain_end {
                domain_end = frag_pos;
            }
        }
        
        let domain = rest[..domain_end].to_string();
        let remaining = &rest[domain_end..];
        
        let (path_with_query, fragment) = if let Some(frag_pos) = remaining.find('#') {
            (&remaining[..frag_pos], Some(remaining[frag_pos + 1..].to_string()))
        } else {
            (remaining, None)
        };
        
        let (path, query_str) = if let Some(query_pos) = path_with_query.find('?') {
            (&path_with_query[..query_pos], Some(&path_with_query[query_pos + 1..]))
        } else {
            (path_with_query, None)
        };
        
        let mut query_params = HashMap::new();
        if let Some(query) = query_str {
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("").to_string();
                    query_params.insert(key.to_string(), value);
                }
            }
        }
        
        Ok(ParsedUrl {
            protocol,
            domain,
            path: path.to_string(),
            query_params,
            fragment,
        })
    }
    
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn has_fragment(&self) -> bool {
        self.fragment.is_some()
    }
    
    pub fn is_secure(&self) -> bool {
        self.protocol == "https"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
        assert!(parsed.is_secure());
    }
    
    #[test]
    fn test_parse_url_without_protocol() {
        let url = "example.com/page";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/page");
        assert!(parsed.query_params.is_empty());
        assert!(!parsed.has_fragment());
    }
    
    #[test]
    fn test_parse_url_with_empty_query_value() {
        let url = "http://test.com/search?q=&sort=desc";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_query_param("q"), Some(&"".to_string()));
        assert_eq!(parsed.get_query_param("sort"), Some(&"desc".to_string()));
    }
}