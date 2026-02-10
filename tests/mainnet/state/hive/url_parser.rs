
use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string()
                );
            }
        }
        
        params
    }
    
    pub fn get_param(query: &str, key: &str) -> Option<String> {
        Self::parse(query).get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query() {
        let query = "name=John%20Doe&age=30&city=New%20York";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
        assert_eq!(params.get("country"), None);
    }
    
    #[test]
    fn test_empty_query() {
        let params = QueryParser::parse("");
        assert!(params.is_empty());
    }
    
    #[test]
    fn test_get_specific_param() {
        let query = "token=abc123&expires=3600";
        let token = QueryParser::get_param(query, "token");
        assert_eq!(token, Some("abc123".to_string()));
        
        let missing = QueryParser::get_param(query, "missing");
        assert_eq!(missing, None);
    }
}