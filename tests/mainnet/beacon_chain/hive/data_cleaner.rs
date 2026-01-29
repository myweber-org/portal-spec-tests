
use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|f| f(entry)))
            .collect()
    }
}

pub fn create_default_cleaner() -> DataCleaner {
    let mut cleaner = DataCleaner::new();

    cleaner.add_filter(|entry| {
        entry.contains_key("id") && !entry.get("id").unwrap().is_empty()
    });

    cleaner.add_filter(|entry| {
        entry.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .map_or(false, |timestamp| timestamp > 0)
    });

    cleaner.add_filter(|entry| {
        !entry.values().any(|value| value.trim().is_empty())
    });

    cleaner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_filters_invalid_data() {
        let cleaner = create_default_cleaner();

        let mut valid_entry = HashMap::new();
        valid_entry.insert("id".to_string(), "123".to_string());
        valid_entry.insert("timestamp".to_string(), "1672531200".to_string());
        valid_entry.insert("name".to_string(), "test".to_string());

        let mut invalid_entry = HashMap::new();
        invalid_entry.insert("id".to_string(), "".to_string());
        invalid_entry.insert("timestamp".to_string(), "0".to_string());

        let data = vec![valid_entry.clone(), invalid_entry];
        let cleaned = cleaner.clean(data);

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0]["id"], "123");
    }

    #[test]
    fn test_empty_filter_chain() {
        let cleaner = DataCleaner::new();
        let mut entry = HashMap::new();
        entry.insert("test".to_string(), "value".to_string());

        let data = vec![entry];
        let cleaned = cleaner.clean(data);

        assert_eq!(cleaned.len(), 1);
    }
}