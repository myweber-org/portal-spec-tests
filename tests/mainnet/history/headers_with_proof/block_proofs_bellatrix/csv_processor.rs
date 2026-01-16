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
    records: Vec<CsvRecord>,
    errors: Vec<CsvError>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line_num = line_num + 1;
            let line = line_result.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_num, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            match self.parse_line(&line, line_num) {
                Ok(record) => {
                    if let Err(validation_err) = self.validate_record(&record) {
                        self.errors.push(validation_err);
                    } else {
                        self.records.push(record);
                    }
                }
                Err(parse_err) => {
                    self.errors.push(parse_err);
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 fields, found {}", parts.len()),
                line_num,
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: {}", parts[0]),
                line_num,
            )
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: {}", parts[2]),
                line_num,
            )
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean format: {}", parts[3]),
                line_num,
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Record ID {} has empty name", record.id),
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Record ID {} has negative value: {}", record.id, record.value),
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn get_errors(&self) -> &[CsvError] {
        &self.errors
    }

    pub fn calculate_statistics(&self) -> Option<Statistics> {
        if self.records.is_empty() {
            return None;
        }

        let mut total_value = 0.0;
        let mut min_value = f64::MAX;
        let mut max_value = f64::MIN;
        let mut active_count = 0;

        for record in &self.records {
            total_value += record.value;
            min_value = min_value.min(record.value);
            max_value = max_value.max(record.value);
            if record.active {
                active_count += 1;
            }
        }

        let avg_value = total_value / self.records.len() as f64;

        Some(Statistics {
            total_records: self.records.len(),
            active_records: active_count,
            total_value,
            average_value: avg_value,
            min_value,
            max_value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub total_records: usize,
    pub active_records: usize,
    pub total_value: f64,
    pub average_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics:\n  Total Records: {}\n  Active Records: {}\n  Total Value: {:.2}\n  Average Value: {:.2}\n  Min Value: {:.2}\n  Max Value: {:.2}",
            self.total_records,
            self.active_records,
            self.total_value,
            self.average_value,
            self.min_value,
            self.max_value
        )
    }
}