use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let without_scheme = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            let domain_end = without_scheme.find('/').unwrap_or(without_scheme.len());
            Some(without_scheme[..domain_end].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://subdomain.example.org:8080/api";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("subdomain.example.org:8080".to_string()));
    }

    #[test]
    fn test_invalid_url_no_scheme() {
        assert_eq!(UrlParser::extract_domain("example.com"), None);
    }
}
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let after_protocol = if url.starts_with("http://") {
            &url[7..]
        } else if url.starts_with("https://") {
            &url[8..]
        } else {
            url
        };
        
        let domain_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let domain = &after_protocol[..domain_end];
        
        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
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
        assert_eq!(
            UrlParser::extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            Some("invalid-url".to_string())
        );
    }
}
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    allowed_schemes: HashSet<String>,
}

impl UrlParser {
    pub fn new(schemes: Vec<&str>) -> Self {
        let allowed_schemes = schemes
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        UrlParser { allowed_schemes }
    }

    pub fn parse(&self, input: &str) -> Result<ParsedUrl, ParseError> {
        let pattern = r"^(?P<scheme>[a-zA-Z][a-zA-Z0-9+\-.]*):(?P<rest>.*)$";
        let re = Regex::new(pattern).map_err(|_| ParseError::InvalidRegex)?;
        
        let caps = re.captures(input).ok_or(ParseError::NoSchemeFound)?;
        let scheme = caps.name("scheme").unwrap().as_str().to_lowercase();
        
        if !self.allowed_schemes.contains(&scheme) {
            return Err(ParseError::SchemeNotAllowed(scheme));
        }
        
        let rest = caps.name("rest").unwrap().as_str();
        if rest.is_empty() {
            return Err(ParseError::EmptyPath);
        }
        
        Ok(ParsedUrl {
            scheme,
            path: rest.to_string(),
            original: input.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub path: String,
    pub original: String,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidRegex,
    NoSchemeFound,
    SchemeNotAllowed(String),
    EmptyPath,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidRegex => write!(f, "Internal regex compilation failed"),
            ParseError::NoSchemeFound => write!(f, "No URL scheme found in input"),
            ParseError::SchemeNotAllowed(scheme) => write!(f, "Scheme '{}' is not allowed", scheme),
            ParseError::EmptyPath => write!(f, "URL path cannot be empty after scheme"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_custom_scheme() {
        let parser = UrlParser::new(vec!["app", "custom", "internal"]);
        let result = parser.parse("app://settings/profile");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.scheme, "app");
        assert_eq!(parsed.path, "//settings/profile");
    }

    #[test]
    fn test_scheme_not_allowed() {
        let parser = UrlParser::new(vec!["https", "http"]);
        let result = parser.parse("ftp://example.com");
        assert_eq!(result, Err(ParseError::SchemeNotAllowed("ftp".to_string())));
    }

    #[test]
    fn test_case_insensitive_scheme() {
        let parser = UrlParser::new(vec!["MYAPP"]);
        let result = parser.parse("MyApp://data");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().scheme, "myapp");
    }
}