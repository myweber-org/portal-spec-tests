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
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }

    pub fn clean(&mut self) -> Vec<String> {
        let mut unique_set = HashSet::new();
        let mut cleaned = Vec::new();

        for record in &self.records {
            let normalized = record.trim().to_lowercase();
            if unique_set.insert(normalized.clone()) {
                cleaned.push(record.trim().to_string());
            }
        }

        cleaned
    }

    pub fn get_stats(&self) -> (usize, usize) {
        let original_count = self.records.len();
        let unique_count = self.records
            .iter()
            .map(|s| s.trim().to_lowercase())
            .collect::<HashSet<_>>()
            .len();
        (original_count, unique_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("Apple");
        cleaner.add_record("apple ");
        cleaner.add_record("Banana");
        cleaner.add_record("banana");

        let cleaned = cleaner.clean();
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_stats(), (4, 2));
    }

    #[test]
    fn test_cleaner_normalizes_whitespace() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  Orange  ");
        cleaner.add_record("Orange");

        let cleaned = cleaner.clean();
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0], "Orange");
    }
}