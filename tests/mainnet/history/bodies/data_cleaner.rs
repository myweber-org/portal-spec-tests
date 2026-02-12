use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Vec<T>>,
}

impl<T> DataCleaner<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<Vec<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self, null_value: &T) {
        self.data.retain(|row| !row.iter().any(|cell| cell == null_value));
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| seen.insert(row.clone()));
    }

    pub fn get_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    pub fn into_data(self) -> Vec<Vec<T>> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let data = vec![
            vec!["A", "B", "C"],
            vec!["null", "E", "F"],
            vec!["G", "H", "null"],
            vec!["I", "J", "K"],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows(&"null");
        assert_eq!(cleaner.get_data().len(), 2);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![1, 2, 3],
            vec![7, 8, 9],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data().len(), 3);
    }
}