use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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
    delimiter: char,
    expected_columns: Option<usize>,
}

impl CsvProcessor {
    pub fn new(delimiter: char) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns: None,
        }
    }

    pub fn with_expected_columns(mut self, columns: usize) -> Self {
        self.expected_columns = Some(columns);
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            let trimmed = line_content.trim();
            if trimmed.is_empty() {
                continue;
            }

            let columns: Vec<String> = trimmed
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(expected) = self.expected_columns {
                if columns.len() != expected {
                    return Err(CsvError::ValidationError(format!(
                        "Line {}: Expected {} columns, found {}",
                        line_number,
                        expected,
                        columns.len()
                    )));
                }
            }

            records.push(CsvRecord { columns });
        }

        if records.is_empty() {
            return Err(CsvError::ParseError("File contains no valid data".to_string()));
        }

        Ok(records)
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Result<Vec<String>, CsvError> {
        let mut result = Vec::with_capacity(records.len());
        
        for (i, record) in records.iter().enumerate() {
            if column_index >= record.columns.len() {
                return Err(CsvError::ValidationError(format!(
                    "Record {}: Column index {} out of bounds (max {})",
                    i,
                    column_index,
                    record.columns.len() - 1
                )));
            }
            result.push(record.columns[column_index].clone());
        }
        
        Ok(result)
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',').with_expected_columns(3);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["name", "age", "city"]);
        assert_eq!(records[1].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_column_extraction() {
        let records = vec![
            CsvRecord { columns: vec!["a".to_string(), "b".to_string()] },
            CsvRecord { columns: vec!["c".to_string(), "d".to_string()] },
        ];
        
        let processor = CsvProcessor::new(',');
        let column = processor.extract_column(&records, 1).unwrap();
        
        assert_eq!(column, vec!["b", "d"]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    FormatError(String),
    ValidationError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::FormatError(msg) => write!(f, "Format error: {}", msg),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ParseError {}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, ParseError> {
    let file = File::open(&path).map_err(|e| {
        ParseError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line.map_err(|e| {
            ParseError::IoError(format!("Failed to read line {}: {}", line_number, e))
        })?;

        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        if fields.len() != 4 {
            return Err(ParseError::FormatError(format!(
                "Line {}: expected 4 fields, found {}",
                line_number,
                fields.len()
            )));
        }

        let id = fields[0].parse::<u32>().map_err(|_| {
            ParseError::ValidationError(format!("Line {}: invalid ID format", line_number))
        })?;

        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::ValidationError(format!(
                "Line {}: name cannot be empty",
                line_number
            )));
        }

        let value = fields[2].parse::<f64>().map_err(|_| {
            ParseError::ValidationError(format!("Line {}: invalid value format", line_number))
        })?;

        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(ParseError::ValidationError(format!(
                    "Line {}: invalid boolean value",
                    line_number
                )))
            }
        };

        records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err(ParseError::ValidationError(
            "No valid records found in file".to_string(),
        ));
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_count = records.iter().filter(|r| r.active).count();
    let total_value: f64 = records.iter().map(|r| r.value).sum();
    let avg_value = if !records.is_empty() {
        total_value / records.len() as f64
    } else {
        0.0
    };

    let max_value = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (avg_value, max_value, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,45.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,32.1,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,ItemC,67.8,yes").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[0].value, 45.5);
        assert!(records[0].active);
        assert!(!records[1].active);
        assert!(records[2].active);
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,45.5").unwrap();

        let result = parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(ParseError::FormatError(_))));
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: false,
            },
            Record {
                id: 3,
                name: "Test3".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let (avg, max, active_count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(max, 30.0);
        assert_eq!(active_count, 2);
    }
}