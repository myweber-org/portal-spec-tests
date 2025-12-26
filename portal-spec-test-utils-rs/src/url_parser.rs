use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() < 2 {
            return None;
        }

        let domain_part = parts[1];
        let domain_end = domain_part.find('/').unwrap_or(domain_part.len());
        let domain = &domain_part[..domain_end];

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
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    params.insert(
                        kv[0].to_string(),
                        kv[1].to_string()
                    );
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

        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };

        if let Some(path_start) = after_protocol.find('/') {
            let path = &after_protocol[path_start..];
            
            if let Some(query_start) = path.find('?') {
                Some(path[..query_start].to_string())
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
            UrlParser::parse_domain("ftp://files.server.org"),
            Some("files.server.org".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/users"),
            Some("/api/users".to_string())
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