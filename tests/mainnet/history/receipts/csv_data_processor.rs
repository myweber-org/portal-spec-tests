
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

fn transform_record(record: &mut Record) {
    record.name = record.name.to_uppercase();
    record.value = (record.value * 100.0).round() / 100.0;
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        if let Err(e) = validate_record(&record) {
            eprintln!("Validation failed: {}", e);
            continue;
        }
        
        transform_record(&mut record);
        writer.serialize(&record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "D".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let mut record = Record {
            id: 1,
            name: "test".to_string(),
            value: 10.12345,
            category: "A".to_string(),
        };
        
        transform_record(&mut record);
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 10.12);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    fn to_csv_string(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }

    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV line format".into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        Ok(Record::new(id, name, value, active))
    }
}

fn read_records_from_file(filename: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record = Record::from_csv_line(&line)?;
        records.push(record);
    }

    Ok(records)
}

fn filter_records_by_value(records: Vec<Record>, threshold: f64) -> Vec<Record> {
    records
        .into_iter()
        .filter(|r| r.value >= threshold && r.active)
        .collect()
}

fn write_records_to_file(records: &[Record], filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    writeln!(file, "id,name,value,active")?;

    for record in records {
        writeln!(file, "{}", record.to_csv_string())?;
    }

    Ok(())
}

fn process_csv_data(input_file: &str, output_file: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let records = read_records_from_file(input_file)?;
    println!("Read {} records from {}", records.len(), input_file);

    let filtered_records = filter_records_by_value(records, threshold);
    println!("Filtered to {} records with value >= {}", filtered_records.len(), threshold);

    write_records_to_file(&filtered_records, output_file)?;
    println!("Written filtered records to {}", output_file);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_filename = "data/input.csv";
    let output_filename = "data/output.csv";
    let threshold = 50.0;

    match process_csv_data(input_filename, output_filename, threshold) {
        Ok(_) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV data: {}", e),
    }

    Ok(())
}use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self { id, category, value, active }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: DataRecord = result?;
            records.push(record);
        }

        Ok(Self { records })
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn export_filtered(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut writer = Writer::from_path(output_path)?;

        for record in filtered {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.5, true),
            DataRecord::new(2, "B".to_string(), 20.0, false),
            DataRecord::new(3, "A".to_string(), 30.5, true),
        ];

        let processor = DataProcessor { records };
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.calculate_average(), 20.333333333333332);
        assert_eq!(processor.get_active_records().len(), 2);
    }
}
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

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            let record = CsvRecord::new(id, name, value, category);
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_all(&self) -> Vec<Result<(), String>> {
        self.records
            .iter()
            .map(|record| record.validate())
            .collect()
    }

    pub fn apply_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
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
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
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

        let category_a_items = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_items.len(), 2);

        processor.apply_transformation(2.0);
        assert_eq!(processor.calculate_total_value(), 1200.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut count = 0;

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total / count as f64))
            .collect()
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

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,category,value,active\n1,electronics,299.99,true\n2,books,19.99,true\n3,electronics,599.99,false\n4,clothing,49.99,true"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_csv() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(csv_file.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(processor.count_records(), 4);
    }

    #[test]
    fn test_filter_by_category() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_csv(csv_file.path()).unwrap();

        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);

        let books = processor.filter_by_category("books");
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, 2);
    }

    #[test]
    fn test_calculate_average() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_csv(csv_file.path()).unwrap();

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        let expected_avg = (299.99 + 19.99 + 599.99 + 49.99) / 4.0;
        assert!((avg.unwrap() - expected_avg).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_csv(csv_file.path()).unwrap();

        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 3);

        let electronics_avg = aggregates
            .iter()
            .find(|(cat, _)| cat == "electronics")
            .map(|(_, avg)| *avg)
            .unwrap();
        let expected_electronics_avg = (299.99 + 599.99) / 2.0;
        assert!((electronics_avg - expected_electronics_avg).abs() < 0.001);
    }
}