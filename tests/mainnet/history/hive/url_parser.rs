use regex::Regex;
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
        if let Some(query_start) = url.find('?') {
            let query_str = &url[query_start + 1..];
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
        let url = "https://www.example.com/path?key=value";
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
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
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
        self.url.host_str()
    }

    pub fn path_segments(&self) -> Vec<&str> {
        self.url.path_segments()
            .map(|segments| segments.collect())
            .unwrap_or_default()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::parse("https://example.com/path?key=value#section").unwrap();
        assert_eq!(parser.scheme(), "https");
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.path_segments(), vec!["path"]);
        assert_eq!(parser.query_params().get("key"), Some(&"value".to_string()));
        assert_eq!(parser.fragment(), Some("section"));
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlParser::parse("not-a-valid-url");
        assert!(result.is_err());
    }
}