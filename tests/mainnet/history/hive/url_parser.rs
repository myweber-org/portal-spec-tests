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
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let params = parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "alice".to_string());
        params.insert("score".to_string(), "95".to_string());
        
        let query = build_query_string(&params);
        assert!(query.contains("name=alice"));
        assert!(query.contains("score=95"));
        assert_eq!(query.matches('&').count(), 1);
    }
}