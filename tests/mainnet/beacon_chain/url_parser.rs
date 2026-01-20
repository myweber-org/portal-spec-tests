
use std::collections::HashMap;

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
        let result = parse_query_string("");
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_parse_single_param() {
        let result = parse_query_string("name=john");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_parse_multiple_params() {
        let result = parse_query_string("name=john&age=30&city=newyork");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
        assert_eq!(result.get("city"), Some(&"newyork".to_string()));
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