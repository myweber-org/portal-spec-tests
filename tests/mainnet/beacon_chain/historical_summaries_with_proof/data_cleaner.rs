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
use regex::Regex;
use std::collections::HashSet;

pub fn clean_and_normalize(input: &str, stopwords: Option<HashSet<&str>>) -> String {
    let re = Regex::new(r"[^\w\s]").unwrap();
    let mut cleaned = re.replace_all(input, "").to_lowercase();
    
    cleaned = cleaned.trim().to_string();
    
    if let Some(stopword_set) = stopwords {
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let filtered: Vec<&str> = words
            .iter()
            .filter(|&&word| !stopword_set.contains(word))
            .copied()
            .collect();
        cleaned = filtered.join(" ");
    }
    
    cleaned
}

pub fn generate_slug(input: &str) -> String {
    let cleaned = clean_and_normalize(input, None);
    cleaned.replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    
    #[test]
    fn test_basic_cleaning() {
        let input = "Hello, World! This is a TEST.";
        let result = clean_and_normalize(input, None);
        assert_eq!(result, "hello world this is a test");
    }
    
    #[test]
    fn test_with_stopwords() {
        let input = "the quick brown fox jumps over the lazy dog";
        let mut stopwords = HashSet::new();
        stopwords.insert("the");
        stopwords.insert("over");
        
        let result = clean_and_normalize(input, Some(stopwords));
        assert_eq!(result, "quick brown fox jumps lazy dog");
    }
    
    #[test]
    fn test_slug_generation() {
        let input = "Rust Programming Language";
        let result = generate_slug(input);
        assert_eq!(result, "rust-programming-language");
    }
}