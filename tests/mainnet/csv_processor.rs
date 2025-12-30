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