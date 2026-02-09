
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut cleaned_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                cleaned_url = &url[prefix.len()..];
                break;
            }
        }

        let end = cleaned_url.find('/').unwrap_or(cleaned_url.len());
        let domain_part = &cleaned_url[..end];

        if domain_part.is_empty() {
            None
        } else {
            Some(domain_part.to_string())
        }
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                if let Some(equal_pos) = pair.find('=') {
                    let key = &pair[..equal_pos];
                    let value = &pair[equal_pos + 1..];
                    
                    if !key.is_empty() {
                        params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        params
    }

    pub fn extract_path(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut cleaned_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                cleaned_url = &url[prefix.len()..];
                break;
            }
        }

        if let Some(slash_pos) = cleaned_url.find('/') {
            let path = &cleaned_url[slash_pos..];
            if let Some(query_pos) = path.find('?') {
                Some(path[..query_pos].to_string())
            } else {
                Some(path.to_string())
            }
        } else {
            Some("/".to_string())
        }
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
            UrlParser::parse_domain("http://sub.domain.co.uk:8080"),
            Some("sub.domain.co.uk:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params(
            "https://example.com/search?q=rust&lang=en&sort=desc"
        );
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/search?q=test"),
            Some("/search".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    MalformedUrl,
    InvalidEncoding,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MalformedUrl => write!(f, "URL format is invalid"),
            ParseError::InvalidEncoding => write!(f, "URL contains invalid percent encoding"),
        }
    }
}

impl Error for ParseError {}

pub fn parse_query_params(url: &str) -> Result<HashMap<String, String>, ParseError> {
    let query_start = url.find('?').ok_or(ParseError::MalformedUrl)?;
    let query_str = &url[query_start + 1..];
    
    let mut params = HashMap::new();
    
    for pair in query_str.split('&') {
        if pair.is_empty() {
            continue;
        }
        
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap();
        let value = parts.next().unwrap_or("");
        
        let decoded_key = percent_decode(key).map_err(|_| ParseError::InvalidEncoding)?;
        let decoded_value = percent_decode(value).map_err(|_| ParseError::InvalidEncoding)?;
        
        params.insert(decoded_key, decoded_value);
    }
    
    Ok(params)
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let mut decoded = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex1 = chars.next().ok_or(())?.to_digit(16).ok_or(())?;
            let hex2 = chars.next().ok_or(())?.to_digit(16).ok_or(())?;
            let byte = (hex1 << 4 | hex2) as u8;
            decoded.push(byte as char);
        } else if c == '+' {
            decoded.push(' ');
        } else {
            decoded.push(c);
        }
    }
    
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&page=1";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
    }
    
    #[test]
    fn test_encoded_values() {
        let url = "https://example.com/?name=John%20Doe&city=New%20York";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
    }
    
    #[test]
    fn test_empty_value() {
        let url = "https://example.com/?flag&empty=";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("flag"), Some(&"".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }
    
    #[test]
    fn test_invalid_url() {
        let url = "https://example.com/search";
        let result = parse_query_params(url);
        
        assert!(matches!(result, Err(ParseError::MalformedUrl)));
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn extract_domain(&self) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/:]+)").unwrap();
        re.captures(&self.url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_re = Regex::new(r"[?&]([^=]+)=([^&]+)").unwrap();

        for cap in query_re.captures_iter(&self.url) {
            if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                params.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let url_re = Regex::new(r"^(https?://)?[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$").unwrap();
        url_re.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://www.example.com/path?key=value");
        assert_eq!(parser.extract_domain(), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://example.com?name=john&age=30");
        let params = parser.parse_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid_parser = UrlParser::new("https://example.com");
        let invalid_parser = UrlParser::new("not-a-valid-url");
        assert!(valid_parser.is_valid_url());
        assert!(!invalid_parser.is_valid_url());
    }
}