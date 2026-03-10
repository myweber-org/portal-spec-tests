use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let pattern = r"^(https?|ftp)://[^\s/$.?#].[^\s]*$";
    let re = Regex::new(pattern).unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://www.rust-lang.org"));
        assert!(is_valid_url("ftp://files.example.com/data.txt"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("://example.com"));
    }
}