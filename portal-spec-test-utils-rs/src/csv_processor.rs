use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if record.is_empty() {
                return Err(CsvError::ParseError(
                    format!("Empty record at line {}", line_number)
                ));
            }

            if self.has_header && line_number == 1 {
                continue;
            }

            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid data records found".to_string()
            ));
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "Record set cannot be empty".to_string()
            ));
        }

        let expected_columns = records[0].len();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(CsvError::ValidationError(
                    format!("Record {} has {} columns, expected {}", 
                           index + 1, record.len(), expected_columns)
                ));
            }

            for (col_index, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(CsvError::ValidationError(
                        format!("Empty field at record {}, column {}", 
                               index + 1, col_index + 1)
                    ));
                }
            }
        }

        Ok(())
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Result<f64, CsvError> {
    if records.is_empty() {
        return Err(CsvError::ValidationError("No records to process".to_string()));
    }

    if column_index >= records[0].len() {
        return Err(CsvError::ValidationError(
            format!("Column index {} out of bounds", column_index)
        ));
    }

    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if let Ok(value) = record[column_index].parse::<f64>() {
            sum += value;
            count += 1;
        } else {
            return Err(CsvError::ParseError(
                format!("Failed to parse value '{}' as number", record[column_index])
            ));
        }
    }

    if count == 0 {
        return Err(CsvError::ValidationError(
            "No valid numeric values found in specified column".to_string()
        ));
    }

    Ok(sum / count as f64)
}