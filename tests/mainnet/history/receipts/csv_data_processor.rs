
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