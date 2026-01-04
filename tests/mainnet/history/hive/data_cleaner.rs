
use std::collections::HashMap;

pub struct DataCleaner {
    data: HashMap<String, Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            data: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, column_name: &str, values: Vec<Option<String>>) {
        self.data.insert(column_name.to_string(), values);
    }

    pub fn clean_data(&mut self) -> HashMap<String, Vec<String>> {
        let mut cleaned = HashMap::new();
        
        for (column, values) in &self.data {
            let cleaned_values: Vec<String> = values
                .iter()
                .filter_map(|val| {
                    val.as_ref().and_then(|s| {
                        let trimmed = s.trim().to_string();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    })
                })
                .collect();
            
            cleaned.insert(column.clone(), cleaned_values);
        }
        
        cleaned
    }

    pub fn count_valid_entries(&self) -> usize {
        self.data
            .values()
            .flat_map(|values| values.iter())
            .filter(|val| val.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.add_column(
            "names",
            vec![
                Some("  John  ".to_string()),
                Some("".to_string()),
                None,
                Some("Alice".to_string()),
                Some("  Bob  ".to_string()),
            ],
        );

        let cleaned = cleaner.clean_data();
        let names = cleaned.get("names").unwrap();
        
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "John");
        assert_eq!(names[1], "Alice");
        assert_eq!(names[2], "Bob");
        assert_eq!(cleaner.count_valid_entries(), 4);
    }
}