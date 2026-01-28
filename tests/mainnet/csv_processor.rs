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
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| CsvError::IoError(e.to_string()))?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: expected 4 fields, got {}", line_num + 1, fields.len())
            ));
        }

        let id = fields[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid ID format", line_num + 1)
            ))?;

        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: name cannot be empty", line_num + 1)
            ));
        }

        let value = fields[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid value format", line_num + 1)
            ))?;

        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: invalid boolean value", line_num + 1)
            )),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError("No valid records found".to_string()));
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_count = records.iter().filter(|r| r.active).count();
    
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / records.len() as f64;
    
    (avg, variance.sqrt(), active_count)
}
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let record = CsvRecord {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                active: parts[3].parse()?,
            };
            
            self.records.push(record);
        }
        
        Ok(self.records.len())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold && record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_to_writer<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "ID,Name,Value,Active")?;
        for record in &self.records {
            writeln!(writer, "{},{},{},{}", record.id, record.name, record.value, record.active)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_csv_processing() {
        let mut processor = CsvProcessor::new();
        let test_data = "1,Alice,42.5,true\n2,Bob,30.0,false\n3,Charlie,55.7,true\n";
        let cursor = Cursor::new(test_data.as_bytes());
        
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(test_data.as_bytes()).unwrap();
        
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let filtered = processor.filter_by_value(40.0);
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 42.73).abs() < 0.01);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column: column,
            filter_value: value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut processed_count = 0;
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    pub fn transform_column(&self, transform_column: usize, transform_fn: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut transformed_count = 0;
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let mut columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > transform_column {
                let original_value = columns[transform_column];
                let transformed_value = transform_fn(original_value);
                columns[transform_column] = &transformed_value;
                transformed_count += 1;
                
                let new_line = columns.join(",");
                writeln!(output_file, "{}", new_line)?;
            } else {
                writeln!(output_file, "{}", line)?;
            }
        }
        
        Ok(transformed_count)
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        fs::write(test_input, test_data).unwrap();
        
        let processor = CsvProcessor::new(test_input, test_output, 2, "active");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("alice"));
        assert!(!output_content.contains("bob"));
        assert!(output_content.contains("charlie"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }
}

fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Line {} failed: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_records: Vec<&Record> = records.iter().filter(|r| r.active).collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len();
    let average = sum / count as f64;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;

    (average, variance.sqrt(), count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Loaded {} total records", records.len());
    
    let (mean, std_dev, active_count) = calculate_statistics(&records);
    println!("Active records: {}", active_count);
    println!("Mean value: {:.2}", mean);
    println!("Standard deviation: {:.2}", std_dev);
    
    let max_record = records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    
    if let Some(record) = max_record {
        println!("Highest value record: ID {}, Name: {}", record.id, record.name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let line = "42,Test Item,123.45,true";
        let record = Record::from_csv_line(line).unwrap();
        
        assert_eq!(record.id, 42);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.value, 123.45);
        assert!(record.active);
    }

    #[test]
    fn test_invalid_record() {
        let line = "42,Test Item";
        let result = Record::from_csv_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (mean, std_dev, count) = calculate_statistics(&records);
        assert_eq!(count, 2);
        assert_eq!(mean, 15.0);
        assert!(std_dev > 7.07 && std_dev < 7.08);
    }

    #[test]
    fn test_file_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "# Test data")?;
        writeln!(temp_file, "1,Alpha,100.5,true")?;
        writeln!(temp_file, "2,Bravo,200.0,false")?;
        writeln!(temp_file, "3,Charlie,300.75,true")?;
        
        let records = process_csv_file(temp_file.path())?;
        assert_eq!(records.len(), 3);
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            skip_header: false,
        }
    }
}

impl CsvConfig {
    pub fn new(delimiter: char, selected_columns: Vec<usize>, skip_header: bool) -> Self {
        CsvConfig {
            delimiter,
            selected_columns,
            skip_header,
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
        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let processed_row = self.process_line(&line);
            results.push(processed_row);
        }

        Ok(results)
    }

    fn process_line(&self, line: &str) -> Vec<String> {
        let parts: Vec<&str> = line.split(self.config.delimiter).collect();
        
        if self.config.selected_columns.is_empty() {
            parts.iter().map(|&s| s.to_string()).collect()
        } else {
            self.config.selected_columns
                .iter()
                .filter_map(|&idx| parts.get(idx).map(|&s| s.to_string()))
                .collect()
        }
    }

    pub fn summarize(&self, data: &[Vec<String>]) -> (usize, usize) {
        let row_count = data.len();
        let col_count = if row_count > 0 { data[0].len() } else { 0 };
        (row_count, col_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_processing() {
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "a,b,c\n1,2,3\n4,5,6").unwrap();
        
        let result = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "2", "3"]);
    }

    #[test]
    fn test_column_selection() {
        let config = CsvConfig::new(',', vec![0, 2], false);
        let processor = CsvProcessor::new(config);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "a,b,c\n1,2,3\n4,5,6").unwrap();
        
        let result = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(result[0], vec!["1", "3"]);
        assert_eq!(result[1], vec!["4", "6"]);
    }

    #[test]
    fn test_summarize() {
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let (rows, cols) = processor.summarize(&data);
        assert_eq!(rows, 2);
        assert_eq!(cols, 2);
    }
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

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub fn process_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&file_path).map_err(|e| {
        CsvError::IoError(format!("Failed to open file {}: {}", file_path.as_ref().display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line.map_err(|e| {
            CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
        })?;

        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        if fields.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                fields.len()
            )));
        }

        let id = fields[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, fields[0]))
        })?;

        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = fields[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!(
                "Line {}: Invalid value format '{}'",
                line_number, fields[2]
            ))
        })?;

        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(CsvError::ParseError(format!(
                    "Line {}: Invalid boolean value '{}'",
                    line_number, fields[3]
                )))
            }
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "CSV file contains no valid records".to_string(),
        ));
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_count = records.iter().filter(|r| r.active).count();
    let total_value: f64 = records.iter().map(|r| r.value).sum();
    let average_value = if !records.is_empty() {
        total_value / records.len() as f64
    } else {
        0.0
    };

    (total_value, average_value, active_count)
}