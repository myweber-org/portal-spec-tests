
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

    pub fn clean_data(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|filter| filter(entry)))
            .collect()
    }
}

pub fn validate_email(entry: &HashMap<String, String>) -> bool {
    if let Some(email) = entry.get("email") {
        email.contains('@') && email.contains('.')
    } else {
        false
    }
}

pub fn validate_age(entry: &HashMap<String, String>) -> bool {
    if let Some(age_str) = entry.get("age") {
        if let Ok(age) = age_str.parse::<u8>() {
            return age >= 18 && age <= 120;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_filter(validate_email);
        cleaner.add_filter(validate_age);

        let test_data = vec![
            [("email".to_string(), "user@example.com".to_string()), ("age".to_string(), "25".to_string())]
                .iter().cloned().collect(),
            [("email".to_string(), "invalid-email".to_string()), ("age".to_string(), "30".to_string())]
                .iter().cloned().collect(),
            [("email".to_string(), "another@test.org".to_string()), ("age".to_string(), "15".to_string())]
                .iter().cloned().collect(),
        ];

        let cleaned = cleaner.clean_data(test_data);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].get("email").unwrap(), "user@example.com");
    }
}