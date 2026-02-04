use regex::Regex;
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
        let re = Regex::new(r"https?://([^/]+)").unwrap();
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
        let url_re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        url_re.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://www.example.com/path?query=value");
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
        let invalid_parser = UrlParser::new("not-a-url");
        
        assert!(valid_parser.is_valid_url());
        assert!(!invalid_parser.is_valid_url());
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    EmptyQuery,
}

pub fn parse_query_string(query: &str) -> Result<HashMap<String, String>, ParseError> {
    if query.is_empty() {
        return Err(ParseError::EmptyQuery);
    }

    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        
        match (parts.next(), parts.next()) {
            (Some(key), Some(value)) => {
                if key.is_empty() {
                    return Err(ParseError::InvalidFormat);
                }
                params.insert(key.to_string(), value.to_string());
            }
            _ => return Err(ParseError::InvalidFormat),
        }
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_query() {
        let query = "name=john&age=25&city=new+york";
        let result = parse_query_string(query).unwrap();
        
        let mut expected = HashMap::new();
        expected.insert("name".to_string(), "john".to_string());
        expected.insert("age".to_string(), "25".to_string());
        expected.insert("city".to_string(), "new+york".to_string());
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_query() {
        let query = "";
        let result = parse_query_string(query);
        assert_eq!(result, Err(ParseError::EmptyQuery));
    }

    #[test]
    fn test_parse_invalid_format() {
        let query = "name=john&age";
        let result = parse_query_string(query);
        assert_eq!(result, Err(ParseError::InvalidFormat));
    }

    #[test]
    fn test_parse_with_empty_key() {
        let query = "=value&key=test";
        let result = parse_query_string(query);
        assert_eq!(result, Err(ParseError::InvalidFormat));
    }
}use std::collections::HashMap;
use url::Url;

pub struct UrlParser {
    url: Url,
}

impl UrlParser {
    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(input)?;
        Ok(Self { url })
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.domain()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn path(&self) -> &str {
        self.url.path()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    pub fn is_secure(&self) -> bool {
        self.url.scheme() == "https"
    }

    pub fn has_query(&self) -> bool {
        self.url.query().is_some()
    }

    pub fn build_absolute_url(&self, relative_path: &str) -> Result<String, url::ParseError> {
        let joined = self.url.join(relative_path)?;
        Ok(joined.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::parse("https://example.com/path?key=value#fragment").unwrap();
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.scheme(), "https");
        assert_eq!(parser.path(), "/path");
        assert!(parser.is_secure());
        assert!(parser.has_query());
        assert_eq!(parser.fragment(), Some("fragment"));
    }

    #[test]
    fn test_query_params() {
        let parser = UrlParser::parse("https://example.com?name=john&age=30").unwrap();
        let params = parser.query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_joining() {
        let parser = UrlParser::parse("https://example.com/base/").unwrap();
        let absolute = parser.build_absolute_url("subpath").unwrap();
        assert_eq!(absolute, "https://example.com/base/subpath");
    }
}
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Option<ParsedUrl> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let (scheme, rest) = Self::extract_scheme(url);
        let (host, path_and_query) = Self::extract_host(rest)?;
        let (path, query) = Self::split_path_and_query(path_and_query);
        let query_params = Self::parse_query_string(query);

        Some(ParsedUrl {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path: path.to_string(),
            query_params,
        })
    }

    fn extract_scheme(url: &str) -> (&str, &str) {
        if let Some(pos) = url.find("://") {
            (&url[..pos], &url[pos + 3..])
        } else {
            ("https", url)
        }
    }

    fn extract_host(rest: &str) -> Option<(&str, &str)> {
        let end = rest.find('/').unwrap_or(rest.len());
        let host = &rest[..end];
        if host.is_empty() {
            return None;
        }
        let remaining = if end < rest.len() { &rest[end..] } else { "" };
        Some((host, remaining))
    }

    fn split_path_and_query(path_and_query: &str) -> (&str, Option<&str>) {
        let query_start = path_and_query.find('?');
        match query_start {
            Some(pos) => (&path_and_query[..pos], Some(&path_and_query[pos + 1..])),
            None => (path_and_query, None),
        }
    }

    fn parse_query_string(query: Option<&str>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(query_str) = query {
            for pair in query_str.split('&') {
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
}

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn domain(&self) -> Option<&str> {
        let parts: Vec<&str> = self.host.split('.').collect();
        if parts.len() >= 2 {
            Some(parts[parts.len() - 2])
        } else {
            None
        }
    }

    pub fn tld(&self) -> Option<&str> {
        self.host.split('.').last()
    }

    pub fn has_query_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.query_params.len(), 2);
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_url_without_scheme() {
        let url = "example.com/api";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/api");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_without_path() {
        let url = "https://subdomain.example.co.uk";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "subdomain.example.co.uk");
        assert_eq!(parsed.path, "");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_domain_extraction() {
        let parsed = UrlParser::parse("https://www.google.com/search").unwrap();
        assert_eq!(parsed.domain(), Some("google"));
        assert_eq!(parsed.tld(), Some("com"));
    }

    #[test]
    fn test_empty_url() {
        assert!(UrlParser::parse("").is_none());
        assert!(UrlParser::parse("   ").is_none());
    }

    #[test]
    fn test_url_with_only_host() {
        let parsed = UrlParser::parse("localhost:8080").unwrap();
        assert_eq!(parsed.host, "localhost:8080");
        assert_eq!(parsed.path, "");
    }
}