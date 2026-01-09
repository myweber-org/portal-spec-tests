
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub timestamp: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }

        groups
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            total_records: self.records.len(),
            average_value: self.calculate_average(),
            categories_count: self.group_by_category().len(),
        }
    }
}

#[derive(Debug)]
pub struct DataSummary {
    pub total_records: usize,
    pub average_value: Option<f64>,
    pub categories_count: usize,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,timestamp").unwrap();
        writeln!(file, "1,electronics,299.99,2023-01-15").unwrap();
        writeln!(file, "2,clothing,49.95,2023-01-16").unwrap();
        writeln!(file, "3,electronics,599.99,2023-01-17").unwrap();
        writeln!(file, "4,books,19.99,2023-01-18").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!(avg > 0.0);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.value, 599.99);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert!(processor.calculate_average().is_none());
        assert!(processor.find_max_value().is_none());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    pub fn to_csv_row(&self) -> String {
        format!("{},{},{:.2},{}", self.id, self.name, self.value, self.category)
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
        
        let category = parts[3].trim().to_string();

        match Record::new(id, name, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Validation error at line {}: {}", line_num + 1, e).into()),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = Record::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let invalid_record = Record::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "Test".to_string(), 100.0, "A".to_string()).unwrap();
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();

        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            Record::new(2, "B".to_string(), 20.0, "X".to_string()).unwrap(),
            Record::new(3, "C".to_string(), 30.0, "Y".to_string()).unwrap(),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_category_filter() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            Record::new(2, "B".to_string(), 20.0, "Y".to_string()).unwrap(),
            Record::new(3, "C".to_string(), 30.0, "X".to_string()).unwrap(),
        ];

        let filtered = filter_by_category(records, "X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X"));
    }
}