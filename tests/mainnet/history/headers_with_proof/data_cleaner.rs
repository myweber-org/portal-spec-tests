use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn clean(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut cleaned = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                cleaned.push(record);
            }
        }

        cleaned
    }

    pub fn filter_by_prefix(&self, prefix: &str) -> Vec<String> {
        self.records
            .iter()
            .filter(|record| record.starts_with(prefix))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple".to_string());
        cleaner.add_record("banana".to_string());
        cleaner.add_record("apple".to_string());
        
        let cleaned = cleaner.clean();
        assert_eq!(cleaned.len(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_filtering() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple".to_string());
        cleaner.add_record("apricot".to_string());
        cleaner.add_record("banana".to_string());
        
        let filtered = cleaner.filter_by_prefix("ap");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"apple".to_string()));
        assert!(filtered.contains(&"apricot".to_string()));
    }
}