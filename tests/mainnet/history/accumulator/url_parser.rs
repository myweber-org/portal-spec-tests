
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        if params.is_empty() { None } else { Some(params) }
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let stripped = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        
        let domain_end = stripped.find('/').unwrap_or(stripped.len());
        let domain = &stripped[..domain_end];
        
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
        let url = "https://example.com/search?q=rust&lang=en&sort=desc";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
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
    }
}
use std::collections::HashMap;
use url::Url;

pub struct UrlParser {
    url: Url,
}

impl UrlParser {
    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(input)?;
        Ok(UrlParser { url })
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.host_str()
    }

    pub fn path(&self) -> &str {
        self.url.path()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
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
        .and_then(|parser| parser.domain().map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::parse("https://example.com/path?key=value#section").unwrap();
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.path(), "/path");
        assert_eq!(parser.scheme(), "https");
        assert_eq!(parser.fragment(), Some("section"));
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

    #[test]
    fn test_url_joining() {
        let parser = UrlParser::parse("https://example.com/base/").unwrap();
        let absolute = parser.build_absolute_url("relative/path").unwrap();
        assert_eq!(absolute, "https://example.com/base/relative/path");
    }
}