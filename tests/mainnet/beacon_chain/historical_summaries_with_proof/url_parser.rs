
use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    let protocol = captures.name("protocol")?.as_str().to_string();
    let domain = captures.name("domain")?.as_str().to_string();
    let path = captures.name("path").map_or("/", |m| m.as_str()).to_string();

    Some(ParsedUrl {
        protocol,
        domain,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let url = "http://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let url = "https://example.com";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_ftp_url() {
        let url = "ftp://files.example.com/pub/data.txt";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.domain, "files.example.com");
        assert_eq!(parsed.path, "/pub/data.txt");
    }

    #[test]
    fn test_invalid_url_returns_none() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}