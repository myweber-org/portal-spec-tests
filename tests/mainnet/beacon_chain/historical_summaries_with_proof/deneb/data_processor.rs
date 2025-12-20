
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.update_category_stats(&record);
        self.records.push(record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        let valid_categories = vec!["A", "B", "C", "D"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(DataError::UnknownCategory);
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn find_highest_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
        self.recalculate_stats();
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_id() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 200.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 150.0, category: "B".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let stats_a = processor.get_category_stats("A").unwrap();
        assert_eq!(stats_a.record_count, 2);
        assert_eq!(stats_a.total_value, 300.0);
        assert_eq!(stats_a.average_value, 150.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "C".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);
        
        let updated_record = &processor.records[0];
        assert_eq!(updated_record.value, 200.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    valid_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(valid_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            valid_categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if !self.valid_categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::InvalidId);
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }
        
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let avg = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let processor = DataProcessor::new(categories);
        
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
    }

    #[test]
    fn test_add_and_retrieve_record() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 42,
            name: "Sample".to_string(),
            value: 250.5,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.get_record(42).unwrap().name, "Sample");
    }

    #[test]
    fn test_calculate_average() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average(), 20.0);
    }
}