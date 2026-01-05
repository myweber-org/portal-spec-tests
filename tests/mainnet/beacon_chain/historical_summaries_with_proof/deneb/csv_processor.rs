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
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
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
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line_content.trim().is_empty() || line_content.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            self.validate_record(&record, line_number)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_num,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_num, parts[0], e))
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(format!(
                "Line {}: Name cannot be empty",
                line_num
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|e| {
            CsvError::ParseError(format!(
                "Line {}: Invalid value '{}': {}",
                line_num, parts[2], e
            ))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(CsvError::ParseError(format!(
                    "Line {}: Invalid boolean value '{}'",
                    line_num, parts[3]
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

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.id == 0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: ID cannot be zero",
                line_num
            )));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative",
                line_num
            )));
        }

        if record.name.len() > 100 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name exceeds maximum length of 100 characters",
                line_num
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

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1,Alice,42.5,true").unwrap();
        writeln!(file, "2,Bob,100.0,false").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "3,Charlie,75.3,yes").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 217.8);
    }

    #[test]
    fn test_invalid_csv_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid,Alice,42.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_errors() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "0,Alice,42.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(file.path());
        assert!(result.is_err());
    }
}