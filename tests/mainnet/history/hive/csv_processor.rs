
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn filter_and_transform_records(
    input_path: &str,
    output_path: &str,
    min_value: f64,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let transformed_record = Record {
                id: record.id,
                name: record.name.to_uppercase(),
                value: record.value * 1.1,
                active: record.active,
            };
            writer.serialize(transformed_record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let threshold = 50.0;

    match filter_and_transform_records(input_file, output_file, threshold) {
        Ok(()) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String, usize),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line: {}", e))
            })?;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line_content.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        self.validate_records(&records)?;
        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len()),
                line_number,
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: {}", parts[0]),
                line_number,
            )
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(
                "Name cannot be empty".to_string(),
                line_number,
            ));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: {}", parts[2]),
                line_number,
            )
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean format: {}", parts[3]),
                line_number,
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in file".to_string(),
            ));
        }

        let mut seen_ids = std::collections::HashSet::new();
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(CsvError::ValidationError(
                    format!("Duplicate ID found: {}", record.id),
                ));
            }

            if record.value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Negative value not allowed for ID: {}", record.id),
                ));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_content = "id,name,value,active\n1,Item1,10.5,true\n2,Item2,20.0,false\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[0].value, 10.5);
        assert!(records[0].active);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    has_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            has_header: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.config.has_header {
            lines.next();
        }

        let mut result = Vec::new();
        for line in lines {
            let line = line?;
            let fields: Vec<String> = line.split(self.config.delimiter).map(String::from).collect();
            
            if self.config.selected_columns.is_empty() {
                result.push(fields);
            } else {
                let selected: Vec<String> = self.config.selected_columns
                    .iter()
                    .filter_map(|&idx| fields.get(idx).cloned())
                    .collect();
                result.push(selected);
            }
        }
        
        Ok(result)
    }

    pub fn filter_rows<F>(&self, data: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        data.into_iter().filter(|row| predicate(row)).collect()
    }
}

pub fn create_sample_data() -> Vec<Vec<String>> {
    vec![
        vec!["Alice".to_string(), "25".to_string(), "Engineer".to_string()],
        vec!["Bob".to_string(), "30".to_string(), "Designer".to_string()],
        vec!["Charlie".to_string(), "35".to_string(), "Manager".to_string()],
    ]
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn filter_and_transform_records(input_path: &str, output_path: &str, min_age: u8) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= min_age && record.active {
            let transformed_record = Record {
                name: record.name.to_uppercase(),
                ..record
            };
            writer.serialize(transformed_record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let minimum_age = 25;
    
    filter_and_transform_records(input_file, output_file, minimum_age)?;
    
    println!("Processing completed successfully");
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_average_value(records: &[CsvRecord]) -> Option<f64> {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return None;
    }

    let total: f64 = active_records.iter()
        .map(|r| r.value)
        .sum();
    
    Some(total / active_records.len() as f64)
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter()
        .filter(|r| r.active)
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,true").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 37.8);
        assert!(!records[1].active);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];
        
        let avg = calculate_average_value(&records).unwrap();
        assert_eq!(avg, 15.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Low".to_string(), value: 5.0, active: true },
            CsvRecord { id: 2, name: "High".to_string(), value: 50.0, active: true },
            CsvRecord { id: 3, name: "Inactive".to_string(), value: 100.0, active: false },
        ];
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 50.0);
    }
}