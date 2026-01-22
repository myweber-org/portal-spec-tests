use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in items {
            if self.seen.insert(item.clone()) {
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

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_numbers() {
        let mut cleaner = DataCleaner::new();
        let numbers = vec![1, 2, 2, 3, 4, 4, 5];
        let deduped = cleaner.deduplicate(numbers);
        assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deduplicate_strings() {
        let mut cleaner = DataCleaner::new();
        let strings = vec!["apple", "banana", "apple", "orange"]
            .into_iter()
            .map(String::from)
            .collect();
        let deduped = cleaner.deduplicate(strings);
        assert_eq!(deduped.len(), 3);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "".to_string(),
            "  ".to_string(),
        ];
        let normalized = DataCleaner::normalize_strings(input);
        assert_eq!(normalized, vec!["hello", "world"]);
    }
}