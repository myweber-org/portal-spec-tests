use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return None;
    }

    let protocol = parts[0].to_string();
    let rest = parts[1];

    let domain_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let domain = domain_path_split[0].to_string();

    let path_and_query = if domain_path_split.len() > 1 {
        domain_path_split[1]
    } else {
        ""
    };

    let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    let path = if !path_query_split[0].is_empty() {
        format!("/{}", path_query_split[0])
    } else {
        "/".to_string()
    };

    let mut query_params = HashMap::new();
    if path_query_split.len() > 1 {
        for pair in path_query_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Some(ParsedUrl {
        protocol,
        domain,
        path,
        query_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "http://test.org/api?key=value&sort=desc";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "test.org");
        assert_eq!(parsed.path, "/api");
        assert_eq!(parsed.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_url_no_path() {
        let url = "ftp://fileserver.net";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.domain, "fileserver.net");
        assert_eq!(parsed.path, "/");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                params.insert(key, value);
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                return Some(after_protocol[..end].to_string());
            }
            return Some(after_protocol.to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("www.example.com".to_string()));
    }

    #[test]
    fn test_extract_domain_no_path() {
        let url = "https://api.service.net";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("api.service.net".to_string()));
    }

    #[test]
    fn test_extract_domain_no_protocol() {
        let url = "invalid-url";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, None);
    }
}
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn extract_domain_parts(&self) -> Option<(String, String)> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let tld = parts.last().unwrap().to_string();
            let name = parts[parts.len() - 2].to_string();
            Some((name, tld))
        } else {
            None
        }
    }

    pub fn get_query_value(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

#[derive(Debug)]
pub enum UrlParseError {
    InvalidFormat,
    MissingScheme,
    MissingDomain,
}

impl FromStr for ParsedUrl {
    type Err = UrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scheme = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let parts: Vec<&str> = s.split("://").collect();
        if parts.len() != 2 {
            return Err(UrlParseError::MissingScheme);
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let fragment_split: Vec<&str> = rest.split('#').collect();
        let before_fragment = fragment_split[0];
        if fragment_split.len() > 1 {
            fragment = Some(fragment_split[1].to_string());
        }

        let query_split: Vec<&str> = before_fragment.split('?').collect();
        let path_and_domain = query_split[0];
        if query_split.len() > 1 {
            for param in query_split[1].split('&') {
                let pair: Vec<&str> = param.split('=').collect();
                if pair.len() == 2 {
                    query_params.insert(pair[0].to_string(), pair[1].to_string());
                }
            }
        }

        let domain_split: Vec<&str> = path_and_domain.split('/').collect();
        if domain_split.is_empty() {
            return Err(UrlParseError::MissingDomain);
        }

        domain = domain_split[0].to_string();
        if domain.is_empty() {
            return Err(UrlParseError::MissingDomain);
        }

        if domain_split.len() > 1 {
            path = domain_split[1..].join("/");
        }

        Ok(ParsedUrl {
            scheme,
            domain,
            path,
            query_params,
            fragment,
        })
    }
}

pub fn parse_url(url: &str) -> Result<ParsedUrl, UrlParseError> {
    ParsedUrl::from_str(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "path/to/resource");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.service.com/data?page=2&limit=50&sort=desc";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "api.service.com");
        assert_eq!(parsed.path, "data");
        assert_eq!(parsed.get_query_value("page"), Some(&"2".to_string()));
        assert_eq!(parsed.get_query_value("limit"), Some(&"50".to_string()));
        assert_eq!(parsed.get_query_value("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_extract_domain_parts() {
        let url = "https://subdomain.example.co.uk/path";
        let parsed = parse_url(url).unwrap();
        let domain_parts = parsed.extract_domain_parts().unwrap();
        
        assert_eq!(domain_parts.0, "co");
        assert_eq!(domain_parts.1, "uk");
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let url = "https://docs.rs/regex/1.5.4/regex/#syntax";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "docs.rs");
        assert_eq!(parsed.path, "regex/1.5.4/regex/");
        assert_eq!(parsed.fragment, Some("syntax".to_string()));
    }
}