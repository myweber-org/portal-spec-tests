
use regex::Regex;

pub fn clean_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_alphanumeric() {
        assert_eq!(clean_alphanumeric("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_alphanumeric("Test@#$%^&*()String"), "TestString");
        assert_eq!(clean_alphanumeric("123_456-789"), "123456789");
        assert_eq!(clean_alphanumeric(""), "");
    }
}