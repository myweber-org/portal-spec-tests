
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_whitespace: bool,
    pub trim_strings: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            normalize_whitespace: true,
            trim_strings: true,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.trim_strings {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
        }

        if self.normalize_whitespace {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data
    }

    pub fn clean_with_options(
        &self,
        data: Vec<String>,
        remove_duplicates: bool,
        normalize_whitespace: bool,
        trim_strings: bool,
    ) -> Vec<String> {
        let mut processor = DataCleaner {
            remove_duplicates,
            normalize_whitespace,
            trim_strings,
        };
        processor.clean_dataset(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_cleaner_normalizes_whitespace() {
        let cleaner = DataCleaner::new();
        let data = vec!["  hello    world  ".to_string(), "data\tprocessing".to_string()];
        let cleaned = cleaner.clean_dataset(data);
        assert!(cleaned.contains(&"hello world".to_string()));
        assert!(cleaned.contains(&"data processing".to_string()));
    }

    #[test]
    fn test_cleaner_with_custom_options() {
        let cleaner = DataCleaner::new();
        let data = vec!["  test  ".to_string(), "  test  ".to_string()];
        let cleaned = cleaner.clean_with_options(data, false, true, true);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0], "test");
    }
}