
use std::collections::HashMap;
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
        self.scheme() == "https"
    }

    pub fn build_absolute_url(&self, relative_path: &str) -> Result<String, url::ParseError> {
        let joined = self.url.join(relative_path)?;
        Ok(joined.to_string())
    }
}

pub fn extract_domain_from_url(url_str: &str) -> Option<String> {
    UrlParser::parse(url_str)
        .ok()
        .and_then(|parser| parser.domain().map(|s| s.to_string()))
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
        assert_eq!(parser.fragment(), Some("fragment"));
        assert!(parser.is_secure());
    }

    #[test]
    fn test_query_params() {
        let parser = UrlParser::parse("https://example.com?name=john&age=30").unwrap();
        let params = parser.query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_domain_extraction() {
        let domain = extract_domain_from_url("https://sub.example.co.uk/path").unwrap();
        assert_eq!(domain, "sub.example.co.uk");
    }
}