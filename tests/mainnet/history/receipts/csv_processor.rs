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
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record, line_num + 1)?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
            ));
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_num, parts[0], e)
            ))?;
        
        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_num, parts[2], e)
            ))?;
        
        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid boolean '{}': {}", line_num, parts[3], e)
            ))?;
        
        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }
        
        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative: {}", line_num, record.value)
            ));
        }
        
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|record| record.id == id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _header = lines.next().transpose()?;
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .into_iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn extract_column(&self, records: Vec<Vec<String>>, column_index: usize) -> Vec<String> {
        records
            .into_iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Carol,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            vec!["A".to_string(), "10".to_string()],
            vec!["B".to_string(), "20".to_string()],
            vec!["C".to_string(), "30".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |record| {
            record.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0) > 15
        });

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "B");
        assert_eq!(filtered[1][0], "C");
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["X".to_string(), "100".to_string(), "Active".to_string()],
            vec!["Y".to_string(), "200".to_string(), "Inactive".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        let column = processor.extract_column(records, 1);

        assert_eq!(column, vec!["100", "200"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
}

fn parse_csv_line(line: &str) -> Result<Record, Box<dyn Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 3 {
        return Err("Invalid number of fields".into());
    }

    let id = parts[0].parse::<u32>()?;
    let name = parts[1].to_string();
    let value = parts[2].parse::<f64>()?;

    Ok(Record { id, name, value })
}

fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.is_empty() {
            continue;
        }

        match parse_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_total(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Processed {} records", records.len());
    println!("Total value: {:.2}", calculate_total(&records));
    
    for record in records.iter().take(3) {
        println!("{:?}", record);
    }
    
    Ok(())
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
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(value) = parts.get(self.filter_column) {
                if value.trim() == self.filter_value {
                    let transformed_line = self.transform_line(&parts);
                    writeln!(output_file, "{}", transformed_line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    fn transform_line(&self, parts: &[&str]) -> String {
        parts
            .iter()
            .map(|s| s.trim().to_uppercase())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        std::fs::write(input_path, test_input).unwrap();
        
        let processor = CsvProcessor::new(input_path, output_path, 2, "active");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(output_path)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("1,ALICE,ACTIVE"));
        assert!(output_content.contains("3,CHARLIE,ACTIVE"));
        assert!(!output_content.contains("BOB"));
        
        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
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
        
        let active = parts[3].parse::<bool>()
            .map_err(|e| format!("Invalid boolean at line {}: {}", line_num + 1, e))?;

        let record = Record::new(id, name, value, active);
        
        if let Err(e) = record.validate() {
            return Err(format!("Validation failed at line {}: {}", line_num + 1, e).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
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
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,true").unwrap();

        let records = parse_csv(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 37.8);
        assert!(!records[1].active);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let result = parse_csv(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            Record::new(1, "Test1".to_string(), 10.0, true),
            Record::new(2, "Test2".to_string(), 20.0, false),
            Record::new(3, "Test3".to_string(), 30.0, true),
        ];
        
        let total = calculate_total(&records);
        assert_eq!(total, 40.0);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Valid".to_string(), 10.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), 10.0, true);
        assert!(invalid_record.validate().is_err());

        let negative_record = Record::new(3, "Test".to_string(), -5.0, true);
        assert!(negative_record.validate().is_err());
    }
}