use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum QueryParam {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

pub struct QueryParser {
    params: HashMap<String, QueryParam>,
}

impl QueryParser {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn parse(&mut self, query: &str) -> Result<(), String> {
        if query.is_empty() {
            return Ok(());
        }

        for pair in query.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid query parameter format: {}", pair));
            }

            let key = parts[0].to_string();
            let value = parts[1];

            let param = self.parse_value(value)?;
            self.params.insert(key, param);
        }

        Ok(())
    }

    fn parse_value(&self, value: &str) -> Result<QueryParam, String> {
        if let Ok(int_val) = i64::from_str(value) {
            return Ok(QueryParam::Integer(int_val));
        }

        if let Ok(float_val) = f64::from_str(value) {
            return Ok(QueryParam::Float(float_val));
        }

        match value.to_lowercase().as_str() {
            "true" => Ok(QueryParam::Boolean(true)),
            "false" => Ok(QueryParam::Boolean(false)),
            _ => Ok(QueryParam::String(value.to_string())),
        }
    }

    pub fn get(&self, key: &str) -> Option<&QueryParam> {
        self.params.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_query() {
        let mut parser = QueryParser::new();
        assert!(parser.parse("").is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn test_parse_single_param() {
        let mut parser = QueryParser::new();
        parser.parse("name=john").unwrap();
        assert_eq!(parser.len(), 1);
        
        if let Some(QueryParam::String(val)) = parser.get("name") {
            assert_eq!(val, "john");
        } else {
            panic!("Expected String parameter");
        }
    }

    #[test]
    fn test_parse_multiple_params() {
        let mut parser = QueryParser::new();
        parser.parse("id=42&active=true&score=98.5").unwrap();
        assert_eq!(parser.len(), 3);

        assert!(matches!(parser.get("id"), Some(QueryParam::Integer(42))));
        assert!(matches!(parser.get("active"), Some(QueryParam::Boolean(true))));
        assert!(matches!(parser.get("score"), Some(QueryParam::Float(98.5))));
    }

    #[test]
    fn test_parse_invalid_format() {
        let mut parser = QueryParser::new();
        let result = parser.parse("key_without_value");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_key() {
        let mut parser = QueryParser::new();
        parser.parse("page=1&limit=10").unwrap();
        assert!(parser.contains_key("page"));
        assert!(parser.contains_key("limit"));
        assert!(!parser.contains_key("offset"));
    }
}