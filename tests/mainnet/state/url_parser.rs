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
}