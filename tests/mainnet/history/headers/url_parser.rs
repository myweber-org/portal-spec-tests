use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MalformedQuery,
    InvalidEncoding,
}

pub fn parse_query_string(query: &str) -> Result<HashMap<String, String>, ParseError> {
    if query.is_empty() {
        return Ok(HashMap::new());
    }

    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        
        let key = parts.next().ok_or(ParseError::MalformedQuery)?;
        let value = parts.next().unwrap_or("");
        
        if key.is_empty() {
            return Err(ParseError::MalformedQuery);
        }
        
        let decoded_key = percent_decode(key).map_err(|_| ParseError::InvalidEncoding)?;
        let decoded_value = percent_decode(value).map_err(|_| ParseError::InvalidEncoding)?;
        
        params.insert(decoded_key, decoded_value);
    }
    
    Ok(params)
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let mut result = Vec::new();
    let mut bytes = input.bytes();
    
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let hex_high = bytes.next().ok_or(())?;
            let hex_low = bytes.next().ok_or(())?;
            
            let decoded = hex_to_byte(hex_high, hex_low).ok_or(())?;
            result.push(decoded);
        } else if byte == b'+' {
            result.push(b' ');
        } else {
            result.push(byte);
        }
    }
    
    String::from_utf8(result).map_err(|_| ())
}

fn hex_to_byte(high: u8, low: u8) -> Option<u8> {
    let to_hex = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    };
    
    let high_val = to_hex(high)?;
    let low_val = to_hex(low)?;
    
    Some((high_val << 4) | low_val)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_query() {
        let result = parse_query_string("").unwrap();
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_single_param() {
        let result = parse_query_string("name=john").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_multiple_params() {
        let result = parse_query_string("name=john&age=25&city=new+york").unwrap();
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"25".to_string()));
        assert_eq!(result.get("city"), Some(&"new york".to_string()));
    }
    
    #[test]
    fn test_percent_encoding() {
        let result = parse_query_string("message=hello%20world%21").unwrap();
        assert_eq!(result.get("message"), Some(&"hello world!".to_string()));
    }
    
    #[test]
    fn test_malformed_query() {
        let result = parse_query_string("=value");
        assert_eq!(result, Err(ParseError::MalformedQuery));
    }
    
    #[test]
    fn test_invalid_encoding() {
        let result = parse_query_string("test=hello%2Gworld");
        assert_eq!(result, Err(ParseError::InvalidEncoding));
    }
}
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut processed_url = url_lower.as_str();
        for prefix in &prefixes {
            if processed_url.starts_with(prefix) {
                processed_url = &processed_url[prefix.len()..];
            }
        }
        
        let domain_end = processed_url.find('/').unwrap_or(processed_url.len());
        let domain = &processed_url[..domain_end];
        
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
                    params.insert(
                        parts[0].to_string(),
                        parts[1].to_string()
                    );
                }
            }
        }
        
        params
    }
    
    pub fn is_valid_url(url: &str) -> bool {
        url.contains("://") && 
        (url.starts_with("http://") || url.starts_with("https://"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_domain() {
        assert_eq!(
            UrlParser::parse_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://subdomain.example.co.uk/"),
            Some("subdomain.example.co.uk".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }
    
    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://localhost:8080"));
        assert!(!UrlParser::is_valid_url("example.com"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}