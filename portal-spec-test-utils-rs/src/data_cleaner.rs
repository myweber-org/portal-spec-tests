use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_text: bool,
    pub remove_empty: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            normalize_text: true,
            remove_empty: true,
        }
    }
    
    pub fn clean_strings(&self, mut data: Vec<String>) -> Vec<String> {
        if self.remove_empty {
            data = filter_empty_strings(data);
        }
        
        if self.normalize_text {
            data = normalize_strings(data);
        }
        
        if self.remove_duplicates {
            data = deduplicate(data);
        }
        
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }
    
    #[test]
    fn test_cleaner_pipeline() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "Banana".to_string(),
            "  banana  ".to_string(),
        ];
        
        let result = cleaner.clean_strings(input);
        assert_eq!(result, vec!["apple".to_string(), "banana".to_string()]);
    }
}