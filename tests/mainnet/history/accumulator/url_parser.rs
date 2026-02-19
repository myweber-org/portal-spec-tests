
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        if params.is_empty() { None } else { Some(params) }
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let stripped = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        
        let domain_end = stripped.find('/').unwrap_or(stripped.len());
        let domain = &stripped[..domain_end];
        
        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&sort=desc";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            UrlParser::extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
    }
}
use std::collections::HashMap;
use url::Url;

pub struct UrlParser {
    url: Url,
}

impl UrlParser {
    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(input)?;
        Ok(UrlParser { url })
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.host_str()
    }

    pub fn path(&self) -> &str {
        self.url.path()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    pub fn is_secure(&self) -> bool {
        self.scheme() == "https"
    }

    pub fn build_absolute_url(&self, relative_path: &str) -> Result<String, url::ParseError> {
        let joined = self.url.join(relative_path)?;
        Ok(joined.to_string())
    }
}

pub fn extract_domain_from_url(url_str: &str) -> Option<String> {
    UrlParser::parse(url_str)
        .ok()
        .and_then(|parser| parser.domain().map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::parse("https://example.com/path?key=value#section").unwrap();
        assert_eq!(parser.domain(), Some("example.com"));
        assert_eq!(parser.path(), "/path");
        assert_eq!(parser.scheme(), "https");
        assert_eq!(parser.fragment(), Some("section"));
        assert!(parser.is_secure());
    }

    #[test]
    fn test_query_params() {
        let parser = UrlParser::parse("https://example.com?name=john&age=30").unwrap();
        let params = parser.query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_domain_extraction() {
        let domain = extract_domain_from_url("https://sub.example.co.uk/path").unwrap();
        assert_eq!(domain, "sub.example.co.uk");
    }

    #[test]
    fn test_url_joining() {
        let parser = UrlParser::parse("https://example.com/base/").unwrap();
        let absolute = parser.build_absolute_url("relative/path").unwrap();
        assert_eq!(absolute, "https://example.com/base/relative/path");
    }
}use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    let protocol = captures.name("protocol")?.as_str().to_string();
    let host = captures.name("host")?.as_str().to_string();
    let path = captures
        .name("path")
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    Some(ParsedUrl { protocol, host, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let url = "http://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let url = "https://example.com";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_ftp_url() {
        let url = "ftp://files.example.com/pub/data.txt";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.host, "files.example.com");
        assert_eq!(parsed.path, "/pub/data.txt");
    }

    #[test]
    fn test_invalid_url_returns_none() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }
        protocol = parts[0].to_string();

        let rest = parts[1];
        let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_split[0];
        let path_and_query = if host_path_split.len() > 1 {
            format!("/{}", host_path_split[1])
        } else {
            "/".to_string()
        };

        let host_port_split: Vec<&str> = authority.split(':').collect();
        host = host_port_split[0].to_string();
        if host_port_split.len() == 2 {
            port = Some(host_port_split[1].parse().map_err(|_| "Invalid port number")?);
        }

        let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
        path = path_query_split[0].to_string();

        if path_query_split.len() == 2 {
            for pair in path_query_split[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            host,
            port,
            path,
            query_params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://www.example.com/index.html";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "www.example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/index.html");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?user=john&page=2";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("user"), Some(&"john".to_string()));
        assert_eq!(parsed.query_params.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not_a_valid_url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}