
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule(record)?;
        }
        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for record in records {
            match self.process_record(record) {
                Ok(processed) => processed_records.push(processed),
                Err(e) => return Err(e),
            }
        }

        Ok(processed_records)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.name.is_empty() {
            Err(ProcessingError::ValidationError("Name cannot be empty".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        if record.value < 0.0 {
            Err(ProcessingError::ValidationError("Value cannot be negative".to_string()))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.name = record.name.to_uppercase();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        let valid_record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.5,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_processing() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "example".to_string(),
            value: 12.3456,
            metadata: HashMap::new(),
        };

        let processed = processor.process_record(record).unwrap();
        assert_eq!(processed.name, "EXAMPLE");
        assert_eq!(processed.value, 12.35);
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.iter().any(|field| field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        let invalid_record = vec!["".to_string(), "field2".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.0".to_string()],
            vec!["15.5".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0);
        
        assert!(stats.is_some());
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 15.333).abs() < 0.001);
        assert!((std_dev - 4.041).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataSet {
    records: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl DataSet {
    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }
        
        Ok(DataSet { records, headers })
    }
    
    pub fn column_stats(&self, column_index: usize) -> Option<ColumnStatistics> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut values = Vec::new();
        let mut frequency_map = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                values.push(value.clone());
                *frequency_map.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        if values.is_empty() {
            return None;
        }
        
        let numeric_values: Vec<f64> = values
            .iter()
            .filter_map(|v| v.parse().ok())
            .collect();
        
        Some(ColumnStatistics {
            column_name: self.headers[column_index].clone(),
            total_count: values.len(),
            unique_count: frequency_map.len(),
            numeric_count: numeric_values.len(),
            frequency_map,
            numeric_values,
        })
    }
    
    pub fn filter_records<F>(&self, predicate: F) -> Vec<&Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool,
    {
        self.records.iter().filter(|r| predicate(r)).collect()
    }
}

pub struct ColumnStatistics {
    column_name: String,
    total_count: usize,
    unique_count: usize,
    numeric_count: usize,
    frequency_map: HashMap<String, usize>,
    numeric_values: Vec<f64>,
}

impl ColumnStatistics {
    pub fn mean(&self) -> Option<f64> {
        if self.numeric_values.is_empty() {
            return None;
        }
        let sum: f64 = self.numeric_values.iter().sum();
        Some(sum / self.numeric_values.len() as f64)
    }
    
    pub fn min_max(&self) -> Option<(f64, f64)> {
        if self.numeric_values.is_empty() {
            return None;
        }
        let min = self.numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        Some((min, max))
    }
    
    pub fn most_frequent(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut items: Vec<_> = self.frequency_map.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));
        items.into_iter().take(n).collect()
    }
    
    pub fn summary(&self) -> String {
        let mut result = format!("Column: {}\n", self.column_name);
        result += &format!("Total values: {}\n", self.total_count);
        result += &format!("Unique values: {}\n", self.unique_count);
        result += &format!("Numeric values: {}\n", self.numeric_count);
        
        if let Some(mean) = self.mean() {
            result += &format!("Mean: {:.2}\n", mean);
        }
        
        if let Some((min, max)) = self.min_max() {
            result += &format!("Range: [{:.2}, {:.2}]\n", min, max);
        }
        
        let top_frequent = self.most_frequent(3);
        if !top_frequent.is_empty() {
            result += "Most frequent values:\n";
            for (value, count) in top_frequent {
                result += &format!("  {}: {}\n", value, count);
            }
        }
        
        result
    }
}

pub fn process_csv_file(input_path: &str) -> Result<String, Box<dyn Error>> {
    let dataset = DataSet::from_csv(input_path)?;
    let mut output = String::new();
    
    output += "Dataset Summary:\n";
    output += &format!("Columns: {}\n", dataset.headers.len());
    output += &format!("Records: {}\n\n", dataset.records.len());
    
    for (i, header) in dataset.headers.iter().enumerate() {
        output += &format!("Analyzing column '{}':\n", header);
        if let Some(stats) = dataset.column_stats(i) {
            output += &stats.summary();
            output += "\n";
        } else {
            output += "  No data available\n\n";
        }
    }
    
    Ok(output)
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

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

        let record = DataRecord::new(id, value, category);
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Skipping invalid record at line {}", line_num + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "A".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "A".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,10.5,CategoryA")?;
        writeln!(temp_file, "2,20.3,CategoryB")?;
        writeln!(temp_file, "# Comment line")?;
        writeln!(temp_file, "3,30.7,CategoryC")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 20.3);
        assert_eq!(records[2].category, "CategoryC");

        Ok(())
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "A".to_string()),
            DataRecord::new(2, 20.0, "B".to_string()),
            DataRecord::new(3, 30.0, "C".to_string()),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
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
        self.records.iter().filter(|r| r.valid).collect()
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
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "B");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,100.0,A").unwrap();
        writeln!(file, "2,200.0,B").unwrap();
        writeln!(file, "3,invalid,C").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "A"));
        processor.records.push(DataRecord::new(2, 200.0, "B"));
        processor.records.push(DataRecord::new(3, 300.0, "A"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(200.0));
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "A"));
        processor.records.push(DataRecord::new(2, 200.0, "B"));
        processor.records.push(DataRecord::new(3, 300.0, "A"));

        let groups = processor.group_by_category();
        assert_eq!(groups.get("A").unwrap().len(), 2);
        assert_eq!(groups.get("B").unwrap().len(), 1);
    }
}use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, Box<dyn Error>> {
        if values.is_empty() {
            return Err("Values vector cannot be empty".into());
        }
        if id == 0 {
            return Err("ID must be non-zero".into());
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn normalize_values(&mut self) {
        let (mean, _, std_dev) = self.calculate_statistics();
        if std_dev > 0.0 {
            for value in &mut self.values {
                *value = (*value - mean) / std_dev;
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<(u32, f64)> {
    records.iter_mut()
        .map(|record| {
            record.normalize_values();
            let (mean, _, _) = record.calculate_statistics();
            (record.id, mean)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 3);
    }
    
    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(0, vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        let (mean, variance, std_dev) = record.calculate_statistics();
        assert!((mean - 2.0).abs() < 1e-10);
        assert!((variance - 0.6666666667).abs() < 1e-10);
        assert!((std_dev - 0.8164965809).abs() < 1e-10);
    }
}