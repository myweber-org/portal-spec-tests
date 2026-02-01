use std::collections::HashSet;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn process(&mut self, item: T) -> Option<T> {
        if self.seen.insert(item.clone()) {
            Some(item)
        } else {
            None
        }
    }

    pub fn process_batch(&mut self, items: Vec<T>) -> Vec<T> {
        items
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }

    pub fn count_unique(&self) -> usize {
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 4, 5];

        let result = cleaner.process_batch(data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.count_unique(), 5);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(cleaner.count_unique(), 3);

        cleaner.reset();
        assert_eq!(cleaner.count_unique(), 0);

        let result = cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(result, vec!["a", "b", "c"]);
    }
}