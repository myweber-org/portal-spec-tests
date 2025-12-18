use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let row_string: String = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"NULL".to_string()))
                .collect::<Vec<&String>>()
                .join("|");
            seen.insert(row_string)
        });
    }

    pub fn get_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }

    pub fn count_rows(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_operations() {
        let mut cleaner = DataCleaner::new(vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
        ]);

        assert_eq!(cleaner.count_rows(), 3);
        
        cleaner.remove_null_rows();
        assert_eq!(cleaner.count_rows(), 2);
        
        cleaner.deduplicate();
        assert_eq!(cleaner.count_rows(), 1);
    }
}