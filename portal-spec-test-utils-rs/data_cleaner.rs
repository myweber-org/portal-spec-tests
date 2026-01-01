
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
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRow {
    pub id: u32,
    pub values: Vec<Option<f64>>,
}

pub struct DataCleaner;

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner
    }

    pub fn clean_dataset(&self, data: &[DataRow]) -> Result<Vec<DataRow>, Box<dyn Error>> {
        let mut cleaned = Vec::new();
        let mut seen_ids = HashSet::new();

        for row in data {
            if seen_ids.contains(&row.id) {
                continue;
            }

            if row.values.iter().all(|v| v.is_some()) {
                seen_ids.insert(row.id);
                cleaned.push(row.clone());
            }
        }

        if cleaned.is_empty() {
            return Err("No valid rows after cleaning".into());
        }

        Ok(cleaned)
    }

    pub fn calculate_statistics(&self, data: &[DataRow]) -> (f64, f64, f64) {
        let mut values = Vec::new();
        for row in data {
            for val in &row.values {
                if let Some(v) = val {
                    values.push(*v);
                }
            }
        }

        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let cleaner = DataCleaner::new();
        let data = vec![
            DataRow { id: 1, values: vec![Some(1.0), Some(2.0)] },
            DataRow { id: 2, values: vec![Some(3.0), None] },
            DataRow { id: 1, values: vec![Some(4.0), Some(5.0)] },
        ];

        let result = cleaner.clean_dataset(&data).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn test_calculate_statistics() {
        let cleaner = DataCleaner::new();
        let data = vec![
            DataRow { id: 1, values: vec![Some(1.0), Some(2.0)] },
            DataRow { id: 2, values: vec![Some(3.0), Some(4.0)] },
        ];

        let (mean, variance, std_dev) = cleaner.calculate_statistics(&data);
        assert!((mean - 2.5).abs() < 1e-10);
        assert!((variance - 1.25).abs() < 1e-10);
        assert!((std_dev - 1.118033988749895).abs() < 1e-10);
    }
}