use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) -> Vec<T> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in self.data.iter() {
            if seen.insert(item) {
                result.push(item.clone());
            }
        }

        self.data = result.clone();
        result
    }

    pub fn get_unique_count(&self) -> usize {
        let set: HashSet<_> = self.data.iter().collect();
        set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 2, 3, 4, 4, 5]);
        let unique = cleaner.remove_duplicates();
        assert_eq!(unique, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.get_unique_count(), 5);
    }

    #[test]
    fn test_empty_data() {
        let cleaner: DataCleaner<i32> = DataCleaner::new(vec![]);
        assert!(cleaner.is_empty());
        assert_eq!(cleaner.get_unique_count(), 0);
    }
}