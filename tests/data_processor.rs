
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
    transformers: Vec<Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformers: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<F>(&mut self, transformer: F)
    where
        F: Fn(HashMap<String, String>) -> HashMap<String, String> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }

    pub fn process(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|record| self.filters.iter().all(|filter| filter(record)))
            .map(|record| {
                let mut transformed = record;
                for transformer in &self.transformers {
                    transformed = transformer(transformed);
                }
                transformed
            })
            .collect()
    }
}

pub fn create_sample_data() -> Vec<HashMap<String, String>> {
    let mut data = Vec::new();
    
    let mut record1 = HashMap::new();
    record1.insert("id".to_string(), "1".to_string());
    record1.insert("name".to_string(), "Alice".to_string());
    record1.insert("age".to_string(), "30".to_string());
    record1.insert("active".to_string(), "true".to_string());
    data.push(record1);
    
    let mut record2 = HashMap::new();
    record2.insert("id".to_string(), "2".to_string());
    record2.insert("name".to_string(), "Bob".to_string());
    record2.insert("age".to_string(), "25".to_string());
    record2.insert("active".to_string(), "false".to_string());
    data.push(record2);
    
    let mut record3 = HashMap::new();
    record3.insert("id".to_string(), "3".to_string());
    record3.insert("name".to_string(), "Charlie".to_string());
    record3.insert("age".to_string(), "35".to_string());
    record3.insert("active".to_string(), "true".to_string());
    data.push(record3);
    
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_filter(|record| {
            record.get("active").map(|v| v == "true").unwrap_or(false)
        });
        
        processor.add_transformer(|mut record| {
            if let Some(age_str) = record.get("age") {
                if let Ok(age) = age_str.parse::<i32>() {
                    let age_group = if age < 30 { "young" } else { "adult" };
                    record.insert("age_group".to_string(), age_group.to_string());
                }
            }
            record
        });
        
        let data = create_sample_data();
        let result = processor.process(data);
        
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|r| r.get("active") == Some(&"true".to_string())));
        assert!(result.iter().all(|r| r.contains_key("age_group")));
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid = self.validate_records().len();
        (total, valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,-5.0,Category1").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 2);
        assert_eq!(processor.calculate_average(), Some(12.75));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Category1").unwrap().len(), 2);
        assert_eq!(groups.get("Category2").unwrap().len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats, (3, 2));
    }
}