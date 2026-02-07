use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    file_path: String,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn aggregate_column(&self, column_index: usize) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut aggregation = HashMap::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if column_index >= parts.len() {
                return Err(format!("Column index {} out of bounds on line {}", column_index, line_num + 1).into());
            }

            let key = parts[column_index].to_string();
            let value: f64 = parts.get(column_index + 1)
                .ok_or_else(|| format!("Missing value column on line {}", line_num + 1))?
                .parse()
                .map_err(|_| format!("Invalid number on line {}", line_num + 1))?;

            *aggregation.entry(key).or_insert(0.0) += value;
        }

        Ok(aggregation)
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<String>, Box<dyn Error>>
    where
        F: Fn(&[&str]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            if predicate(&parts) {
                filtered.push(line);
            }
        }

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aggregation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,5.2").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.aggregate_column(0).unwrap();

        assert_eq!(result.get("A"), Some(&15.7));
        assert_eq!(result.get("B"), Some(&20.3));
    }

    #[test]
    fn test_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,35,London").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let filtered = processor.filter_rows(|parts| parts[2] == "London").unwrap();

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].contains("Alice"));
        assert!(filtered[1].contains("Charlie"));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidHeader,
    MissingColumn(String),
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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if line_num == 0 && self.has_header {
                if record.is_empty() {
                    return Err(CsvError::InvalidHeader);
                }
                continue;
            }

            if record.iter().any(|field| field.is_empty()) {
                return Err(CsvError::ParseError(format!(
                    "Empty field found at line {}",
                    line_num + 1
                )));
            }

            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ParseError("No data records found".to_string()));
        }

        Ok(records)
    }

    pub fn validate_column_count(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(format!(
                    "Column count mismatch at record {}: expected {}, found {}",
                    idx + 1,
                    expected_len,
                    record.len()
                )));
            }
        }
        Ok(())
    }
}

pub fn calculate_column_averages(records: &[Vec<String>]) -> Result<Vec<f64>, CsvError> {
    if records.is_empty() {
        return Err(CsvError::ParseError("No records to process".to_string()));
    }

    let column_count = records[0].len();
    let mut sums = vec![0.0; column_count];
    let mut counts = vec![0; column_count];

    for record in records {
        for (i, field) in record.iter().enumerate() {
            match field.parse::<f64>() {
                Ok(value) => {
                    sums[i] += value;
                    counts[i] += 1;
                }
                Err(_) => {
                    return Err(CsvError::ParseError(format!(
                        "Failed to parse numeric value in column {}: '{}'",
                        i + 1,
                        field
                    )));
                }
            }
        }
    }

    let averages: Vec<f64> = sums
        .iter()
        .zip(counts.iter())
        .map(|(&sum, &count)| {
            if count > 0 {
                sum / count as f64
            } else {
                0.0
            }
        })
        .collect();

    Ok(averages)
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            
            if line_num == 0 && self.has_header {
                self.validate_header(&record)?;
                continue;
            }
            
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Vec<String>, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err(CsvError::ParseError(
                "Empty line encountered".to_string(),
                line_num,
            ));
        }

        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        let mut seen = std::collections::HashSet::new();
        
        for (idx, field) in header.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::InvalidHeader(
                    format!("Header field at position {} is empty", idx + 1)
                ));
            }
            
            if !seen.insert(field) {
                return Err(CsvError::InvalidHeader(
                    format!("Duplicate header field: '{}'", field)
                ));
            }
        }
        
        Ok(())
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

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_invalid_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,name,city").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }
}
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
    records: Vec<CsvRecord>,
    total_value: f64,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            total_value: 0.0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record)?;
            
            self.total_value += record.value;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 columns, found {}", line_num, parts.len())
            ));
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid ID '{}': {}", line_num, parts[0], e)
            ))?;
        
        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid value '{}': {}", line_num, parts[2], e)
            ))?;
        
        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(
                format!("Line {}: Invalid boolean '{}': {}", line_num, parts[3], e)
            ))?;
        
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
                format!("Record {}: Name cannot be empty", record.id)
            ));
        }
        
        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Record {}: Value cannot be negative", record.id)
            ));
        }
        
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn get_total_value(&self) -> f64 {
        self.total_value
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value / self.records.len() as f64)
        }
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}