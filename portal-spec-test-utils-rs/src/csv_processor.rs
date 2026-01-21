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
    valid_count: usize,
    invalid_count: usize,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| {
                CsvError::IoError(format!("Failed to read line: {}", e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            match self.parse_line(&line, line_num + 1) {
                Ok(record) => {
                    if self.validate_record(&record) {
                        self.records.push(record);
                        self.valid_count += 1;
                    } else {
                        self.invalid_count += 1;
                    }
                }
                Err(e) => {
                    self.invalid_count += 1;
                    eprintln!("Warning: {}", e);
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len()),
                line_num
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: '{}'", parts[0]),
                line_num
            )
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: '{}'", parts[2]),
                line_num
            )
        })?;

        let active = parts[3].parse::<bool>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid boolean format: '{}'", parts[3]),
                line_num
            )
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord) -> bool {
        if record.name.is_empty() {
            return false;
        }
        
        if record.value < 0.0 || record.value > 10000.0 {
            return false;
        }

        true
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.valid_count, self.invalid_count)
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut csv_data = "id,name,value,active\n".to_string();
        csv_data.push_str("1,Test1,100.5,true\n");
        csv_data.push_str("2,Test2,200.0,false\n");
        csv_data.push_str("3,Test3,300.75,true\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_stats(), (3, 0));
        assert_eq!(processor.filter_active().len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 200.41666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_invalid_csv() {
        let mut csv_data = "id,name,value,active\n".to_string();
        csv_data.push_str("invalid,Test1,100.5,true\n");
        csv_data.push_str("2,,200.0,false\n");
        csv_data.push_str("3,Test3,-500.0,true\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_stats(), (0, 3));
    }
}