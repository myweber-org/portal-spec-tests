use std::collections::HashSet;

pub struct DataCleaner {
    pub data: Vec<Vec<Option<String>>>,
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
            let key: Vec<String> = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"".to_string()).to_string())
                .collect();
            seen.insert(key)
        });
    }

    pub fn clean(&mut self) {
        self.remove_null_rows();
        self.deduplicate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut raw_data = vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("C".to_string()), Some("3".to_string())],
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        cleaner.clean();

        assert_eq!(cleaner.data.len(), 2);
        assert_eq!(cleaner.data[0][0], Some("A".to_string()));
        assert_eq!(cleaner.data[1][0], Some("C".to_string()));
    }
}