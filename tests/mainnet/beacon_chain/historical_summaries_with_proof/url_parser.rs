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
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut domain = url.to_string();
        for prefix in prefixes {
            if url_lower.starts_with(prefix) {
                domain = url[prefix.len()..].to_string();
                break;
            }
        }

        if let Some(pos) = domain.find('/') {
            domain = domain[..pos].to_string();
        }

        if let Some(pos) = domain.find('?') {
            domain = domain[..pos].to_string();
        }

        if domain.is_empty() {
            None
        } else {
            Some(domain)
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
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path?query=1"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://subdomain.example.co.uk"),
            Some("subdomain.example.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com"),
            Some("example.com".to_string())
        );
        
        assert_eq!(UrlParser::extract_domain(""), None);
    }
}use std::collections::HashMap;

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

pub fn build_query_string(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<String> = Vec::new();
    
    for (key, value) in params {
        pairs.push(format!("{}={}", key, value));
    }
    
    pairs.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty_query() {
        let params = parse_query_string("");
        assert!(params.is_empty());
    }
    
    #[test]
    fn test_parse_single_param() {
        let params = parse_query_string("name=john");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_parse_multiple_params() {
        let params = parse_query_string("name=john&age=30&city=newyork");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
    }
    
    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "john".to_string());
        params.insert("age".to_string(), "30".to_string());
        
        let query = build_query_string(&params);
        assert!(query.contains("name=john"));
        assert!(query.contains("age=30"));
        assert_eq!(query.split('&').count(), 2);
    }
}