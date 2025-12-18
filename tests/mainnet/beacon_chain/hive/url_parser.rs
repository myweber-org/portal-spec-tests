use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let url = if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        url::Url::parse(&url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Ok(parsed_url) = url::Url::parse(url) {
            for (key, value) in parsed_url.query_pairs() {
                params.insert(key.into_owned(), value.into_owned());
            }
        }
        
        params
    }

    pub fn extract_path(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .map(|parsed| parsed.path().to_string())
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
            UrlParser::parse_domain("http://sub.domain.co.uk"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("nonexistent"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/"),
            Some("/".to_string())
        );
    }
}