
use std::collections::HashMap;

pub struct DataCleaner {
    pub remove_nulls: bool,
    pub normalize_strings: bool,
    pub string_normalization_rules: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_nulls: true,
            normalize_strings: false,
            string_normalization_rules: HashMap::new(),
        }
    }

    pub fn clean_dataset<T>(&self, data: Vec<Option<T>>) -> Vec<T>
    where
        T: Clone,
    {
        if self.remove_nulls {
            data.into_iter().filter_map(|x| x).collect()
        } else {
            data.into_iter()
                .map(|x| x.unwrap_or_else(|| panic!("Null value found")))
                .collect()
        }
    }

    pub fn normalize_text(&self, text: &str) -> String {
        if !self.normalize_strings {
            return text.to_string();
        }

        let mut result = text.to_lowercase().trim().to_string();
        
        for (pattern, replacement) in &self.string_normalization_rules {
            result = result.replace(pattern, replacement);
        }

        result
    }

    pub fn add_normalization_rule(&mut self, pattern: &str, replacement: &str) {
        self.string_normalization_rules
            .insert(pattern.to_string(), replacement.to_string());
    }
}

pub fn process_records(records: Vec<Option<String>>) -> Vec<String> {
    let mut cleaner = DataCleaner::new();
    cleaner.normalize_strings = true;
    cleaner.add_normalization_rule("  ", " ");
    cleaner.add_normalization_rule("\t", " ");
    
    let cleaned_data = cleaner.clean_dataset(records);
    
    cleaned_data
        .iter()
        .map(|record| cleaner.normalize_text(record))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let cleaner = DataCleaner::new();
        let data = vec![Some("value1"), None, Some("value2"), Some("value3")];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaned, vec!["value1", "value2", "value3"]);
    }

    #[test]
    fn test_normalize_text() {
        let mut cleaner = DataCleaner::new();
        cleaner.normalize_strings = true;
        cleaner.add_normalization_rule("test", "check");
        
        let result = cleaner.normalize_text("  TEST Example  ");
        assert_eq!(result, "check example");
    }
}