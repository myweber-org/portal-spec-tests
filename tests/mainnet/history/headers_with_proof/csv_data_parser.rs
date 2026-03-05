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

impl CsvRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
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
        
        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = fields[3].parse::<bool>()
            .map_err(|e| format!("Invalid active flag at line {}: {}", line_number, e))?;

        let record = CsvRecord { id, name, value, active };
        
        record.validate()
            .map_err(|e| format!("Validation failed at line {}: {}", line_number, e))?;
        
        records.push(record);
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,John Doe,42.5,true").unwrap();
        writeln!(temp_file, "2,Jane Smith,37.8,false").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "John Doe");
        assert_eq!(records[1].value, 37.8);
    }

    #[test]
    fn test_invalid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,,42.5,true").unwrap();
        
        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ParseError {}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, ParseError> {
        let file = File::open(&path).map_err(|e| {
            ParseError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                ParseError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(ParseError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            ParseError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e))
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|e| {
            ParseError::ParseError(format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e))
        })?;

        if value < 0.0 {
            return Err(ParseError::ValidationError(format!(
                "Line {}: Value cannot be negative: {}",
                line_number, value
            )));
        }

        let active = parts[3].parse::<bool>().map_err(|e| {
            ParseError::ParseError(format!("Line {}: Invalid boolean '{}': {}", line_number, parts[3], e))
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
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
    fn test_csv_parsing() {
        let csv_data = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,20.0,false\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let parser = CsvParser::new(',', true);
        let records = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[0].value, 10.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, variance, std_dev) = CsvParser::calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_validation_errors() {
        let csv_data = "id,name,value,active\n1,,10.5,true\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path());
        
        assert!(result.is_err());
        if let Err(ParseError::ValidationError(msg)) = result {
            assert!(msg.contains("Name cannot be empty"));
        }
    }
}