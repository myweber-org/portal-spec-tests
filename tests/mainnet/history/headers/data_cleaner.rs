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

pub fn filter_by_length(strings: Vec<String>, min_length: usize) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| s.len() >= min_length)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 1, 4];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_filter_by_length() {
        let input = vec!["a".to_string(), "ab".to_string(), "abc".to_string()];
        let result = filter_by_length(input, 2);
        assert_eq!(result, vec!["ab".to_string(), "abc".to_string()]);
    }
}