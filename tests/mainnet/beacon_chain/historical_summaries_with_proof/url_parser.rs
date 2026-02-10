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