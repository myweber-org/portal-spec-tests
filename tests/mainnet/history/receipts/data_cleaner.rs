
use std::collections::HashMap;

pub struct DataCleaner {
    pub remove_nulls: bool,
    pub normalize_strings: bool,
    pub default_values: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_nulls: true,
            normalize_strings: true,
            default_values: HashMap::new(),
        }
    }

    pub fn clean_record(&self, record: &mut HashMap<String, String>) -> Result<(), String> {
        let mut keys_to_remove = Vec::new();

        for (key, value) in record.iter_mut() {
            if value.trim().is_empty() {
                if self.remove_nulls {
                    keys_to_remove.push(key.clone());
                } else if let Some(default_val) = self.default_values.get(key) {
                    *value = default_val.clone();
                }
            } else if self.normalize_strings {
                *value = value.trim().to_lowercase();
            }
        }

        for key in keys_to_remove {
            record.remove(&key);
        }

        Ok(())
    }

    pub fn clean_dataset(&self, dataset: &mut Vec<HashMap<String, String>>) -> Result<(), String> {
        for record in dataset.iter_mut() {
            self.clean_record(record)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_record_removes_null() {
        let cleaner = DataCleaner::new();
        let mut record = HashMap::from([
            ("name".to_string(), "john".to_string()),
            ("age".to_string(), "".to_string()),
            ("city".to_string(), "New York".to_string()),
        ]);

        cleaner.clean_record(&mut record).unwrap();
        assert_eq!(record.len(), 2);
        assert!(record.contains_key("name"));
        assert!(!record.contains_key("age"));
    }

    #[test]
    fn test_clean_record_normalizes_strings() {
        let cleaner = DataCleaner::new();
        let mut record = HashMap::from([
            ("name".to_string(), "  JOHN  ".to_string()),
            ("city".to_string(), "New York".to_string()),
        ]);

        cleaner.clean_record(&mut record).unwrap();
        assert_eq!(record.get("name").unwrap(), "john");
        assert_eq!(record.get("city").unwrap(), "new york");
    }
}