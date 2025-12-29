
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
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
            return Err(format!("Invalid number of columns at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
        
        let active = parts[3].parse::<bool>()
            .map_err(|e| format!("Invalid boolean at line {}: {}", line_num + 1, e))?;

        let record = Record::new(id, name, value, active);
        
        record.validate()
            .map_err(|e| format!("Validation failed at line {}: {}", line_num + 1, e))?;
        
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
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(0, "".to_string(), -5.0, true);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_calculate_total() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];
        
        assert_eq!(calculate_total(&records), 40.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 50.0, true),
            Record::new(3, "C".to_string(), 30.0, false),
        ];
        
        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 50.0);
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

fn validate_record(record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
    if record.id == 0 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: ID must be greater than 0",
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
        writeln!(temp_file, "1,Item One,25.5,true").unwrap();
        writeln!(temp_file, "2,Item Two,100.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Item Three,75.25,yes").unwrap();

        let result = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].name, "Item One");
        assert_eq!(result[0].value, 25.5);
        assert!(result[0].active);
        assert!(!result[1].active);
        assert!(result[2].active);
    }

    #[test]
    fn test_invalid_id() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "0,Invalid Item,10.0,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_missing_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Only Two Fields").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }

    #[test]
    fn test_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }
}