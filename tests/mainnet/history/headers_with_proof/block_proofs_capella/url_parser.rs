use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn new() -> Self {
        QueryParams {
            params: HashMap::new(),
        }
    }

    pub fn parse(query_string: &str) -> Result<Self, String> {
        let mut params = HashMap::new();
        
        if query_string.is_empty() {
            return Ok(QueryParams { params });
        }

        for pair in query_string.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid query parameter format: {}", pair));
            }

            let key = parts[0].to_string();
            let value = parts[1].to_string();

            if key.is_empty() {
                return Err("Query parameter key cannot be empty".to_string());
            }

            params
                .entry(key)
                .or_insert_with(Vec::new)
                .push(value);
        }

        Ok(QueryParams { params })
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.params.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.params
            .entry(key)
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn remove(&mut self, key: &str) -> Option<Vec<String>> {
        self.params.remove(key)
    }

    pub fn to_string(&self) -> String {
        let mut pairs: Vec<String> = Vec::new();
        
        for (key, values) in &self.params {
            for value in values {
                pairs.push(format!("{}={}", key, value));
            }
        }
        
        pairs.join("&")
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_string() {
        let params = QueryParams::parse("").unwrap();
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_parse_single_param() {
        let params = QueryParams::parse("name=john").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("name").unwrap(), &vec!["john".to_string()]);
    }

    #[test]
    fn test_parse_multiple_params() {
        let params = QueryParams::parse("name=john&age=30&city=nyc").unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params.get("name").unwrap(), &vec!["john".to_string()]);
        assert_eq!(params.get("age").unwrap(), &vec!["30".to_string()]);
        assert_eq!(params.get("city").unwrap(), &vec!["nyc".to_string()]);
    }

    #[test]
    fn test_parse_duplicate_keys() {
        let params = QueryParams::parse("color=red&color=blue&color=green").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(
            params.get("color").unwrap(),
            &vec!["red".to_string(), "blue".to_string(), "green".to_string()]
        );
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = QueryParams::parse("name");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid query parameter format"));
    }

    #[test]
    fn test_parse_empty_key() {
        let result = QueryParams::parse("=value");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Query parameter key cannot be empty"));
    }

    #[test]
    fn test_insert_and_remove() {
        let mut params = QueryParams::new();
        params.insert("test".to_string(), "value".to_string());
        
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("test").unwrap(), &vec!["value".to_string()]);
        
        let removed = params.remove("test");
        assert_eq!(removed.unwrap(), vec!["value".to_string()]);
        assert!(params.is_empty());
    }

    #[test]
    fn test_to_string() {
        let mut params = QueryParams::new();
        params.insert("name".to_string(), "john".to_string());
        params.insert("age".to_string(), "30".to_string());
        params.insert("color".to_string(), "red".to_string());
        params.insert("color".to_string(), "blue".to_string());
        
        let query_string = params.to_string();
        assert!(query_string.contains("name=john"));
        assert!(query_string.contains("age=30"));
        assert!(query_string.contains("color=red"));
        assert!(query_string.contains("color=blue"));
        
        let parsed = QueryParams::parse(&query_string).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn test_keys_iterator() {
        let params = QueryParams::parse("a=1&b=2&c=3").unwrap();
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();
        
        assert_eq!(keys, vec!["a", "b", "c"]);
    }
}use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    let protocol = captures.name("protocol")?.as_str().to_string();
    let domain = captures.name("domain")?.as_str().to_string();
    let path = captures
        .name("path")
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    Some(ParsedUrl {
        protocol,
        domain,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let url = "http://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let url = "https://rust-lang.org";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "rust-lang.org");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_ftp_url() {
        let url = "ftp://files.example.com/pub/data.txt";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.domain, "files.example.com");
        assert_eq!(parsed.path, "/pub/data.txt");
    }

    #[test]
    fn test_invalid_url_returns_none() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let after_protocol = if let Some(stripped) = url.strip_prefix("http://") {
            stripped
        } else if let Some(stripped) = url.strip_prefix("https://") {
            stripped
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

    pub fn parse_full(url: &str) -> (Option<String>, HashMap<String, String>) {
        let domain = Self::parse_domain(url);
        let params = Self::parse_query_params(url);
        (domain, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(
            UrlParser::parse_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), Some("invalid-url".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?name=john&age=30");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_parse_full() {
        let (domain, params) = UrlParser::parse_full("https://api.service.com/v1/data?token=abc123&mode=debug");
        assert_eq!(domain, Some("api.service.com".to_string()));
        assert_eq!(params.get("token"), Some(&"abc123".to_string()));
        assert_eq!(params.get("mode"), Some(&"debug".to_string()));
    }
}