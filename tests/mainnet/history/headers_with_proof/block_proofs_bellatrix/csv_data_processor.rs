
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category);
            if let Err(e) = record.validate() {
                eprintln!("Validation error on line {}: {}", line_num + 1, e);
                continue;
            }
            
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,300.0,CategoryA").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 600.0);
        assert_eq!(processor.get_average_value(), 200.0);

        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);

        processor.transform_values(|v| v * 1.1);
        assert_eq!(processor.calculate_total_value(), 660.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            _ => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: AggregationOp) -> Result<f64, String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err(format!("Column '{}' not found", column_name)),
        };

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        match operation {
            AggregationOp::Sum => Ok(numeric_values.iter().sum()),
            AggregationOp::Average => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            AggregationOp::Max => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or("Cannot compute max".into())
                .map(|&v| v),
            AggregationOp::Min => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or("Cannot compute min".into())
                .map(|&v| v),
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub enum AggregationOp {
    Sum,
    Average,
    Max,
    Min,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "Alice,30,50000").unwrap();
        writeln!(file, "Bob,25,45000").unwrap();
        writeln!(file, "Charlie,35,60000").unwrap();
        writeln!(file, "Diana,28,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), &["name", "age", "salary"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_age() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() >= 30);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_aggregate_salary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", AggregationOp::Sum).unwrap();
        assert!((total_salary - 210000.0).abs() < 0.001);
        
        let avg_salary = processor.aggregate_numeric_column("salary", AggregationOp::Average).unwrap();
        assert!((avg_salary - 52500.0).abs() < 0.001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        for result in csv_reader.deserialize() {
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

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn get_active_records(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, category: "A".to_string(), value: 10.5, active: true },
            DataRecord { id: 2, category: "B".to_string(), value: 20.3, active: false },
            DataRecord { id: 3, category: "A".to_string(), value: 15.7, active: true },
        ];

        processor.records = test_data;

        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.calculate_average(), 15.5);
        assert_eq!(processor.get_active_records().len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("A"), Some(&2));
        assert_eq!(counts.get("B"), Some(&1));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        processor.records = vec![
            DataRecord { id: 1, category: "Test".to_string(), value: 42.0, active: true },
        ];
        
        processor.save_filtered_to_csv(temp_path, "Test")?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(temp_path)?;
        
        assert_eq!(new_processor.records.len(), 1);
        assert_eq!(new_processor.records[0].id, 1);
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record::new(id, name, value, category));
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "Alpha".to_string()),
            Record::new(2, "ItemB".to_string(), 20.3, "Beta".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Alpha".to_string()),
        ];

        let filtered = filter_by_category(&records, "Alpha");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.0, "Cat1".to_string()),
            Record::new(2, "ItemB".to_string(), 20.0, "Cat2".to_string()),
            Record::new(3, "ItemC".to_string(), 30.0, "Cat1".to_string()),
        ];

        let avg = calculate_average(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "Alpha".to_string()),
            Record::new(2, "ItemB".to_string(), 25.3, "Beta".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Alpha".to_string()),
        ];

        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.3);
    }
}