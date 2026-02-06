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

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.dedupe_set.contains(&normalized) {
            true
        } else {
            self.dedupe_set.insert(normalized);
            false
        }
    }

    pub fn clean_data(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
            }
        }
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  TEST  "), "test");
        assert_eq!(cleaner.normalize_string("MixedCase"), "mixedcase");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "Apple".to_string(),
            "banana".to_string(),
            "  apple  ".to_string(),
        ];
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn remove_nulls(self) -> Self
    where
        T: PartialEq,
    {
        let filtered_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| *item != None.into())
            .collect();
        Self {
            data: filtered_data,
        }
    }

    pub fn deduplicate(self) -> Self
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let unique_set: HashSet<T> = self.data.into_iter().collect();
        let unique_data: Vec<T> = unique_set.into_iter().collect();
        Self { data: unique_data }
    }

    pub fn get_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_dataset<T>(data: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone + PartialEq,
{
    let cleaner = DataCleaner::new(data);
    cleaner.remove_nulls().deduplicate().get_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let input = vec![
            Some(1),
            None,
            Some(2),
            Some(1),
            None,
            Some(3),
            Some(2),
        ];
        
        let cleaned: Vec<Option<i32>> = clean_dataset(input);
        assert_eq!(cleaned, vec![Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn test_remove_nulls() {
        let cleaner = DataCleaner::new(vec![Some("a"), None, Some("b"), None]);
        let result = cleaner.remove_nulls().get_data();
        assert_eq!(result, vec![Some("a"), Some("b")]);
    }

    #[test]
    fn test_deduplicate() {
        let cleaner = DataCleaner::new(vec![1, 2, 2, 3, 1, 4]);
        let result = cleaner.deduplicate().get_data();
        let mut sorted_result = result;
        sorted_result.sort();
        assert_eq!(sorted_result, vec![1, 2, 3, 4]);
    }
}