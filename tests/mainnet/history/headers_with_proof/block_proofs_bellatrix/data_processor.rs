
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_values_length(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Values array cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero-sum values".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_timestamp(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let processing_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ProcessingError::TransformationError("System time error".to_string()))?
        .as_secs() as i64;

    let mut metadata = record.metadata;
    metadata.insert("processed_at".to_string(), processing_time.to_string());

    Ok(DataRecord {
        metadata,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_success() {
        let record = create_test_record();
        assert!(validate_timestamp(&record).is_ok());
        assert!(validate_values_length(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let mut record = create_test_record();
        record.timestamp = -1;
        assert!(validate_timestamp(&record).is_err());

        record.timestamp = 1625097600;
        record.values.clear();
        assert!(validate_values_length(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let record = create_test_record();
        let normalized = normalize_values(record).unwrap();
        let sum: f64 = normalized.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_processor_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_validation_rule(validate_values_length);
        processor.add_transformation(normalize_values);
        processor.add_transformation(add_processing_timestamp);

        let record = create_test_record();
        let result = processor.process(record);

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_at"));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_stats(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,alpha,1234567890").unwrap();
        writeln!(temp_file, "2,99.9,beta,1234567891").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string(), 1));
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string(), 2));
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string(), 3));

        let (min, max, avg) = processor.get_stats();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                values.push(value);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.count(), 0);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(1.0);
        ds.add_value(2.0);
        ds.add_value(3.0);
        ds.add_value(4.0);
        ds.add_value(5.0);

        assert_eq!(ds.mean(), Some(3.0));
        assert_eq!(ds.variance(), Some(2.5));
        assert_eq!(ds.standard_deviation(), Some(2.5_f64.sqrt()));
        assert_eq!(ds.min(), Some(1.0));
        assert_eq!(ds.max(), Some(5.0));
        assert_eq!(ds.count(), 5);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        writeln!(temp_file, "invalid")?;
        writeln!(temp_file, "25.1")?;

        let ds = DataSet::from_csv(temp_file.path())?;
        assert_eq!(ds.count(), 4);
        assert!((ds.mean().unwrap() - 17.9).abs() < 0.0001);
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}: {:?}", index + 1, fields).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        let values: Vec<f64> = data
            .iter()
            .filter_map(|row| row.get(column_index))
            .filter_map(|s| s.parse().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        Some(sum / count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["30".to_string(), "40".to_string()],
            vec!["50".to_string(), "60".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let avg = processor.calculate_statistics(&data, 0);
        
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 30.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
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

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = Vec::new();
        for record in &self.records {
            if !categories.contains(&record.category) {
                categories.push(record.category.clone());
            }
        }
        categories.sort();
        categories
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 15.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,10.5,TypeA\n");
        csv_content.push_str("2,20.3,TypeB\n");
        csv_content.push_str("3,-5.0,TypeC\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string()));
        processor.records.push(DataRecord::new(3, -5.0, "C".to_string()));

        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn validate_file(&self) -> Result<bool, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let metadata = path.metadata()?;
        if metadata.len() == 0 {
            return Err("File is empty".into());
        }

        Ok(true)
    }

    pub fn process_records<F>(&self, mut callback: F) -> Result<usize, Box<dyn Error>>
    where
        F: FnMut(Vec<String>),
    {
        self.validate_file()?;

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut record_count = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                callback(fields);
                record_count += 1;
            }
        }

        Ok(record_count)
    }

    pub fn calculate_statistics(&self) -> Result<ProcessingStats, Box<dyn Error>> {
        let mut stats = ProcessingStats::new();
        let mut field_counts = Vec::new();

        self.process_records(|fields| {
            stats.total_records += 1;
            field_counts.push(fields.len());
        })?;

        if !field_counts.is_empty() {
            stats.average_fields = field_counts.iter().sum::<usize>() as f64 / field_counts.len() as f64;
            stats.min_fields = *field_counts.iter().min().unwrap_or(&0);
            stats.max_fields = *field_counts.iter().max().unwrap_or(&0);
        }

        Ok(stats)
    }
}

pub struct ProcessingStats {
    pub total_records: usize,
    pub average_fields: f64,
    pub min_fields: usize,
    pub max_fields: usize,
}

impl ProcessingStats {
    pub fn new() -> Self {
        ProcessingStats {
            total_records: 0,
            average_fields: 0.0,
            min_fields: 0,
            max_fields: 0,
        }
    }

    pub fn display(&self) {
        println!("Total records processed: {}", self.total_records);
        println!("Average fields per record: {:.2}", self.average_fields);
        println!("Minimum fields in record: {}", self.min_fields);
        println!("Maximum fields in record: {}", self.max_fields);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_validation() {
        let temp_file = NamedTempFile::new().unwrap();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');

        assert!(processor.validate_file().is_ok());
    }

    #[test]
    fn test_process_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "field1,field2,field3").unwrap();
        writeln!(temp_file, "data1,data2,data3").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let mut record_count = 0;

        let result = processor.process_records(|fields| {
            assert_eq!(fields.len(), 3);
            record_count += 1;
        });

        assert!(result.is_ok());
        assert_eq!(record_count, 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "a,b,c").unwrap();
        writeln!(temp_file, "x,y").unwrap();
        writeln!(temp_file, "1,2,3,4").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let stats = processor.calculate_statistics().unwrap();

        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.min_fields, 2);
        assert_eq!(stats.max_fields, 4);
    }
}