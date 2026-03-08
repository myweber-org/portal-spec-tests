use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(url).map(|caps| caps[1].to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_string = &url[start + 1..];
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        re.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=value";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let url_no_protocol = "example.com/path";
        assert_eq!(UrlParser::parse_domain(url_no_protocol), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://sub.domain.co.uk/path"));
        assert!(!UrlParser::is_valid_url("not-a-url"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn build(params: &HashMap<String, String>) -> String {
        let mut query_parts: Vec<String> = Vec::new();
        
        for (key, value) in params {
            if !key.is_empty() {
                query_parts.push(format!("{}={}", key, value));
            }
        }
        
        query_parts.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = QueryParser::parse("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_single_param() {
        let result = QueryParser::parse("name=john");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_parse_multiple_params() {
        let result = QueryParser::parse("name=john&age=30&city=nyc");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
        assert_eq!(result.get("city"), Some(&"nyc".to_string()));
    }

    #[test]
    fn test_build_empty() {
        let params: HashMap<String, String> = HashMap::new();
        let result = QueryParser::build(&params);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_single_param() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "john".to_string());
        let result = QueryParser::build(&params);
        assert!(result == "name=john");
    }

    #[test]
    fn test_round_trip() {
        let original = "name=john&age=30&city=nyc";
        let parsed = QueryParser::parse(original);
        let rebuilt = QueryParser::build(&parsed);
        assert_eq!(rebuilt, original);
    }
}