use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize<F>(&mut self, normalizer: F) -> &mut Self
    where
        F: Fn(&T) -> T,
    {
        for item in &mut self.data {
            *item = normalizer(item);
        }
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(|item| predicate(item));
        self
    }

    pub fn get_data(&self) -> Vec<T> {
        self.data.clone()
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }
}

pub fn process_strings(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner
        .deduplicate()
        .normalize(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .get_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.count(), 3);
    }

    #[test]
    fn test_string_processing() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "hello".to_string(),
            "  ".to_string(),
            "WORLD".to_string(),
        ];
        let result = process_strings(strings);
        assert_eq!(result, vec!["hello", "world"]);
    }
}