
use regex::Regex;
use std::collections::HashSet;

pub fn clean_and_normalize(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let normalized_spaces = re_multispace.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-\.]").unwrap();
    let cleaned = re_special.replace_all(&normalized_spaces, "");
    
    cleaned.to_lowercase()
}

pub fn remove_duplicates(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_normalize() {
        assert_eq!(
            clean_and_normalize("  Hello   WORLD!!  "),
            "hello world"
        );
    }

    #[test]
    fn test_remove_duplicates() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = remove_duplicates(input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
    }
}