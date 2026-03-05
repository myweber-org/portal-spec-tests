use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^https?://([^/]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(query_start) = url.find('?') {
            let query_str = &url[query_start + 1..];
            for pair in query_str.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?key=value";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30";
        let params = UrlParser::parse_query_params(url);
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}use std::collections::HashMap;
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

    pub fn path_segments(&self) -> Vec<&str> {
        self.url.path_segments()
            .map(|segments| segments.collect())
            .unwrap_or_default()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::parse("https://example.com/path?key=value#section").unwrap();
        assert_eq!(parser.scheme(), "https");
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.path_segments(), vec!["path"]);
        assert_eq!(parser.query_params().get("key"), Some(&"value".to_string()));
        assert_eq!(parser.fragment(), Some("section"));
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlParser::parse("not-a-valid-url");
        assert!(result.is_err());
    }
}use std::collections::HashMap;

pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn parse(query: &str) -> Result<Self, &'static str> {
        if query.is_empty() {
            return Ok(Self {
                params: HashMap::new(),
            });
        }

        let mut params = HashMap::new();
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().ok_or("Missing key in query pair")?;
            let value = parts.next().unwrap_or("");
            
            if key.is_empty() {
                return Err("Empty key in query parameter");
            }
            
            params
                .entry(key.to_string())
                .or_insert_with(Vec::new)
                .push(value.to_string());
        }
        
        Ok(Self { params })
    }
    
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.params.get(key).and_then(|values| values.first()).map(|s| s.as_str())
    }
    
    pub fn get_all(&self, key: &str) -> Option<&[String]> {
        self.params.get(key).map(|values| values.as_slice())
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
        let params = QueryParams::parse("").unwrap();
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }
    
    #[test]
    fn test_parse_single_param() {
        let params = QueryParams::parse("name=john").unwrap();
        assert_eq!(params.get_first("name"), Some("john"));
        assert_eq!(params.len(), 1);
    }
    
    #[test]
    fn test_parse_multiple_values() {
        let params = QueryParams::parse("color=red&color=blue").unwrap();
        assert_eq!(params.get_all("color"), Some(&["red".to_string(), "blue".to_string()][..]));
        assert_eq!(params.get_first("color"), Some("red"));
    }
    
    #[test]
    fn test_parse_without_value() {
        let params = QueryParams::parse("flag&name=test").unwrap();
        assert_eq!(params.get_first("flag"), Some(""));
        assert_eq!(params.get_first("name"), Some("test"));
    }
    
    #[test]
    fn test_invalid_empty_key() {
        let result = QueryParams::parse("=value");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_contains_key() {
        let params = QueryParams::parse("page=1&sort=desc").unwrap();
        assert!(params.contains_key("page"));
        assert!(params.contains_key("sort"));
        assert!(!params.contains_key("limit"));
    }
}