
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn with_filter(mut self, column: usize, value: &str) -> Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut processed_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let fields: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let should_include = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < fields.len() => fields[col] == val,
                _ => true,
            };

            if should_include {
                writeln!(output_file, "{}", line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }
}

pub fn transform_data(input: &str) -> String {
    input
        .trim()
        .split(',')
        .map(|field| field.to_uppercase())
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_transform_data() {
        let input = "hello,world,rust";
        let expected = "HELLO|WORLD|RUST";
        assert_eq!(transform_data(input), expected);
    }

    #[test]
    fn test_csv_processor() -> Result<(), Box<dyn Error>> {
        let test_input = "test_data.csv";
        let test_output = "test_output.csv";

        let content = "id,name,value\n1,apple,100\n2,banana,200\n3,apple,150\n";
        fs::write(test_input, content)?;

        let processor = CsvProcessor::new(test_input, test_output)
            .with_filter(1, "apple");

        let processed = processor.process()?;
        assert_eq!(processed, 2);

        let output_content = fs::read_to_string(test_output)?;
        assert!(output_content.contains("1,apple,100"));
        assert!(output_content.contains("3,apple,150"));
        assert!(!output_content.contains("banana"));

        fs::remove_file(test_input)?;
        fs::remove_file(test_output)?;

        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !columns.is_empty() && !columns.iter().all(|c| c.is_empty()) {
                records.push(CsvRecord { columns });
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn print_records(&self, records: &[CsvRecord]) {
        for (i, record) in records.iter().enumerate() {
            println!("Record {}: {:?}", i + 1, record.columns);
        }
    }
}

pub fn process_csv_sample() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    let records = processor.parse_file("data/sample.csv")?;
    
    let filtered = processor.filter_records(records, |record| {
        record.columns.len() >= 3 && !record.columns[2].is_empty()
    });
    
    println!("Filtered records count: {}", filtered.len());
    processor.print_records(&filtered);
    
    Ok(())
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
        writeln!(temp_file, "Charlie,35,").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord { columns: vec!["A".to_string(), "B".to_string()] },
            CsvRecord { columns: vec!["C".to_string()] },
            CsvRecord { columns: vec!["D".to_string(), "E".to_string(), "F".to_string()] },
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |r| r.columns.len() > 1);
        
        assert_eq!(filtered.len(), 2);
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
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

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
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
        })?;

        let reader = BufReader::new(file);
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            self.validate_record(&record, line_number)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e))
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|e| {
            CsvError::ParseError(format!(
                "Line {}: Invalid value '{}': {}",
                line_number, parts[2], e
            ))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(CsvError::ParseError(format!(
                    "Line {}: Invalid boolean value '{}'",
                    line_number, parts[3]
                )))
            }
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
        if record.id == 0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: ID cannot be zero",
                line_number
            )));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative",
                line_number
            )));
        }

        if self.records.iter().any(|r| r.id == record.id) {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Duplicate ID {}",
                line_number, record.id
            )));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut processor = CsvProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,200.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,Charlie,300.75,yes").unwrap();

        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 601.25);
    }

    #[test]
    fn test_validation_error() {
        let mut processor = CsvProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "1,Bob,200.0,false").unwrap();

        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_err());
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

        let record = parse_csv_line(&line_content, line_number)?;
        validate_record(&record, line_number)?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "CSV file contains no valid records".to_string(),
        ));
    }

    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() != 4 {
        return Err(CsvError::ParseError(format!(
            "Line {}: Expected 4 fields, found {}",
            line_number,
            parts.len()
        )));
    }

    let id = parts[0]
        .parse::<u32>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)))?;

    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Name cannot be empty",
            line_number
        )));
    }

    let value = parts[2]
        .parse::<f64>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)))?;

    let active = parts[3]
        .parse::<bool>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid boolean '{}': {}", line_number, parts[3], e)))?;

    Ok(CsvRecord {
        id,
        name,
        value,
        active,
    })
}

fn validate_record(record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
    if record.id == 0 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: ID cannot be zero",
            line_number
        )));
    }

    if record.value < 0.0 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Value cannot be negative",
            line_number
        )));
    }

    if record.name.len() > 100 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Name exceeds maximum length of 100 characters",
            line_number
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,100.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,0.0,true").unwrap();

        let result = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "Alice");
        assert_eq!(result[1].value, 100.0);
        assert_eq!(result[2].id, 3);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }

    #[test]
    fn test_validation_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "0,Invalid,-10.0,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_aggregate(&self, key_column: usize, value_column: usize) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut aggregation = HashMap::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(self.delimiter).collect();
            if parts.len() <= key_column || parts.len() <= value_column {
                eprintln!("Warning: Line {} has insufficient columns", line_num + 1);
                continue;
            }

            let key = parts[key_column].trim().to_string();
            let value_str = parts[value_column].trim();

            match value_str.parse::<f64>() {
                Ok(value) => {
                    let entry = aggregation.entry(key).or_insert(0.0);
                    *entry += value;
                }
                Err(_) => {
                    eprintln!("Warning: Invalid numeric value '{}' on line {}", value_str, line_num + 1);
                }
            }
        }

        Ok(aggregation)
    }

    pub fn print_summary(&self, data: &HashMap<String, f64>) {
        println!("Aggregation Summary:");
        println!("{:<20} {:<15}", "Key", "Total");
        println!("{}", "-".repeat(35));

        let mut sorted_items: Vec<(&String, &f64)> = data.iter().collect();
        sorted_items.sort_by(|a, b| a.0.cmp(b.0));

        for (key, total) in sorted_items {
            println!("{:<20} {:<15.2}", key, total);
        }

        let overall_total: f64 = data.values().sum();
        println!("{}", "-".repeat(35));
        println!("{:<20} {:<15.2}", "Overall Total", overall_total);
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
        writeln!(temp_file, "category,amount").unwrap();
        writeln!(temp_file, "food,25.50").unwrap();
        writeln!(temp_file, "transport,15.75").unwrap();
        writeln!(temp_file, "food,12.25").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.read_and_aggregate(0, 1).unwrap();

        assert_eq!(result.get("food"), Some(&37.75));
        assert_eq!(result.get("transport"), Some(&15.75));
    }
}