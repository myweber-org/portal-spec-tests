use std::str::FromStr;

pub fn filter_numbers<T: FromStr>(items: Vec<String>) -> Vec<T> {
    items
        .into_iter()
        .filter_map(|s| s.parse::<T>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_numbers() {
        let input = vec![
            "42".to_string(),
            "hello".to_string(),
            "3.14".to_string(),
            "world".to_string(),
            "100".to_string(),
        ];
        let result: Vec<i32> = filter_numbers(input);
        assert_eq!(result, vec![42, 100]);
    }
}use regex::Regex;

pub fn extract_numbers(input: &str) -> String {
    let re = Regex::new(r"[^0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers() {
        assert_eq!(extract_numbers("abc123def456"), "123456");
        assert_eq!(extract_numbers("phone: 555-1234"), "5551234");
        assert_eq!(extract_numbers("no digits here"), "");
        assert_eq!(extract_numbers(""), "");
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, input: &str) -> Option<String> {
        if self.dedupe_set.insert(input.to_string()) {
            Some(input.to_string())
        } else {
            None
        }
    }

    pub fn normalize_whitespace(input: &str) -> String {
        input
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn trim_and_lowercase(input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn clean_pipeline(&mut self, input: &str) -> Option<String> {
        let normalized = Self::normalize_whitespace(input);
        let cleaned = Self::trim_and_lowercase(&normalized);
        self.deduplicate(&cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test").is_some());
        assert!(cleaner.deduplicate("test").is_none());
        assert!(cleaner.deduplicate("another").is_some());
    }

    #[test]
    fn test_normalization() {
        assert_eq!(
            DataCleaner::normalize_whitespace("  hello   world  "),
            "hello world"
        );
    }

    #[test]
    fn test_clean_pipeline() {
        let mut cleaner = DataCleaner::new();
        let result = cleaner.clean_pipeline("  Hello   World  ");
        assert_eq!(result, Some("hello world".to_string()));
        
        let duplicate = cleaner.clean_pipeline("  hello   world  ");
        assert!(duplicate.is_none());
    }
}