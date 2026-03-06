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
pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat(String),
    InvalidNumber(String),
    InvalidBoolean(String),
    MissingColumn(usize),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

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
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, ParseError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 4 {
            return Err(ParseError::MissingColumn(parts.len()));
        }

        let id = parts[0].trim().parse::<u32>()
            .map_err(|_| ParseError::InvalidNumber(format!("Line {}: Invalid ID '{}'", line_num, parts[0])))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidFormat(format!("Line {}: Name cannot be empty", line_num)));
        }

        let value = parts[2].trim().parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(format!("Line {}: Invalid value '{}'", line_num, parts[2])))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(ParseError::InvalidBoolean(format!("Line {}: Invalid boolean '{}'", line_num, parts[3]))),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len();
        let mean = sum / count as f64;

        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count as f64;

        let active_count = records.iter().filter(|r| r.active).count();

        (mean, variance, active_count)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
            ParseError::InvalidBoolean(msg) => write!(f, "Invalid boolean: {}", msg),
            ParseError::MissingColumn(count) => write!(f, "Missing columns, found {} but expected 4", count),
        }
    }
}

impl Error for ParseError {}