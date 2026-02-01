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

    pub fn process(&self, mut data: HashMap<String, String>) -> Option<HashMap<String, String>> {
        for filter in &self.filters {
            if !filter(&data) {
                return None;
            }
        }

        for transformer in &self.transformers {
            data = transformer(data);
        }

        Some(data)
    }

    pub fn process_batch(&self, batch: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        batch
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_filter(|data| {
        data.contains_key("id") && !data.get("id").unwrap().is_empty()
    });

    processor.add_transformer(|mut data| {
        data.entry("processed".to_string())
            .or_insert("true".to_string());
        data
    });

    processor.add_transformer(|mut data| {
        if let Some(value) = data.get_mut("name") {
            *value = value.trim().to_string();
        }
        data
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let processor = create_default_processor();
        
        let mut valid_data = HashMap::new();
        valid_data.insert("id".to_string(), "123".to_string());
        valid_data.insert("name".to_string(), "  test  ".to_string());
        
        let result = processor.process(valid_data);
        assert!(result.is_some());
        
        let processed = result.unwrap();
        assert_eq!(processed.get("id").unwrap(), "123");
        assert_eq!(processed.get("name").unwrap(), "test");
        assert_eq!(processed.get("processed").unwrap(), "true");
    }

    #[test]
    fn test_invalid_data() {
        let processor = create_default_processor();
        
        let mut invalid_data = HashMap::new();
        invalid_data.insert("name".to_string(), "test".to_string());
        
        let result = processor.process(invalid_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        
        let mut data1 = HashMap::new();
        data1.insert("id".to_string(), "1".to_string());
        data1.insert("name".to_string(), "  a  ".to_string());
        
        let mut data2 = HashMap::new();
        data2.insert("name".to_string(), "b".to_string());
        
        let mut data3 = HashMap::new();
        data3.insert("id".to_string(), "3".to_string());
        data3.insert("name".to_string(), "  c  ".to_string());
        
        let batch = vec![data1, data2, data3];
        let results = processor.process_batch(batch);
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.get("processed").unwrap() == "true"));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    loaded_count += 1;
                }
                Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
            }
        }

        Ok(loaded_count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn process_with_multiplier(&self, multiplier: f64) -> Vec<f64> {
        self.records
            .iter()
            .map(|r| r.calculate_adjusted_value(multiplier))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -5.0, "test".to_string()).is_err());
        assert!(DataRecord::new(1, 5.0, "".to_string()).is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.5,category_a").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.total_value(), 46.0);
        assert_eq!(processor.average_value(), Some(46.0 / 3.0));
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let processed = processor.process_with_multiplier(2.0);
        assert_eq!(processed, vec![21.0, 40.0, 31.0]);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());

        Ok(processed)
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test", &[1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("invalid", &[f64::NAN, 1.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.process_data("stats", &[1.0, 2.0, 3.0]).unwrap();
        let stats = processor.calculate_statistics("stats");
        assert!(stats.is_some());
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 4.0);
        assert_eq!(variance, 8.0);
        assert_eq!(std_dev, 2.8284271247461903);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process_with_filter<F>(&self, filter_fn: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_fn(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn calculate_column_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut sum = 0.0;
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split(self.delimiter).collect();

            if column_index < fields.len() {
                if let Ok(value) = fields[column_index].trim().parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_with_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary\nAlice,30,50000\nBob,25,45000\nCharlie,35,60000").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor
            .process_with_filter(|fields| {
                fields.len() > 1 && fields[1].parse::<i32>().unwrap_or(0) > 30
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_calculate_column_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.0\n30.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let average = processor.calculate_column_average(0).unwrap();

        assert!((average - 20.333).abs() < 0.001);
    }
}