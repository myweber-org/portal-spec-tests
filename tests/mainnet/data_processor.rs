use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.3,type_b").unwrap();
        writeln!(temp_file, "3,-5.0,type_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "a"));
        processor.records.push(DataRecord::new(2, 20.0, "b"));
        processor.records.push(DataRecord::new(3, 30.0, "c"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let average = processor.calculate_average();
        assert_eq!(average, None);
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "cat_a"));
        processor.records.push(DataRecord::new(2, 20.0, "cat_a"));
        processor.records.push(DataRecord::new(3, 30.0, "cat_b"));

        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("cat_a").unwrap().len(), 2);
        assert_eq!(groups.get("cat_b").unwrap().len(), 1);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, value: &str) -> bool {
        self.validators
            .get(name)
            .map_or(false, |validator| validator(value))
    }

    pub fn transform(&self, name: &str, value: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(value))
    }

    pub fn process_data(&self, value: &str) -> Option<String> {
        if self.validate("email", value) {
            self.transform("uppercase", value.to_string())
        } else {
            None
        }
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator("email", Box::new(|value| {
        value.contains('@') && value.contains('.')
    }));

    processor.add_validator("numeric", Box::new(|value| {
        value.chars().all(|c| c.is_ascii_digit())
    }));

    processor.add_transformer("uppercase", Box::new(|value| {
        value.to_uppercase()
    }));

    processor.add_transformer("reverse", Box::new(|value| {
        value.chars().rev().collect()
    }));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_numeric_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("numeric", "12345"));
        assert!(!processor.validate("numeric", "123abc"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_process_data() {
        let processor = create_default_processor();
        let result = processor.process_data("test@example.com");
        assert_eq!(result, Some("TEST@EXAMPLE.COM".to_string()));
    }
}