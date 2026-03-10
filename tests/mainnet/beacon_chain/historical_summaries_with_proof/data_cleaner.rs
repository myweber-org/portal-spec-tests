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
        .filter(|s| !s.is_empty())
        .collect()
}

pub struct DataCleaner {
    pub remove_empty: bool,
    pub deduplicate: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_empty: true,
            deduplicate: true,
            normalize_case: true,
        }
    }

    pub fn clean_strings(&self, mut data: Vec<String>) -> Vec<String> {
        if self.normalize_case {
            data = normalize_strings(data);
        }
        
        if self.deduplicate {
            data = deduplicate(data);
        }
        
        if self.remove_empty {
            data.retain(|s| !s.is_empty());
        }
        
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec!["a", "b", "a", "c", "b"];
        let result = deduplicate(input);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "".to_string(),
            "  ".to_string(),
        ];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_data_cleaner() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "Banana".to_string(),
            "  banana  ".to_string(),
        ];
        let result = cleaner.clean_strings(input);
        assert_eq!(result, vec!["apple", "banana"]);
    }
}