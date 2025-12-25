use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
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

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(CsvError::ParseError(
                format!("Line {}: expected 3 columns, found {}", line_num + 1, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid ID format '{}'", line_num + 1, parts[0])
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: name cannot be empty", line_num + 1)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid value format '{}'", line_num + 1, parts[2])
            ))?;

        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: value cannot be negative", line_num + 1)
            ));
        }

        records.push(CsvRecord { id, name, value });
    }

    Ok(records)
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

    (sum, mean, std_dev)
}