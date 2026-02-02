
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid number of fields at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = parts[3].trim().parse::<bool>()
            .map_err(|e| format!("Invalid active flag at line {}: {}", line_number, e))?;

        let record = Record::new(id, name, value, active);
        
        if let Err(e) = record.validate() {
            return Err(format!("Validation error at line {}: {}", line_number, e).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();

    (sum, avg, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,15.2,true").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 33.7);
        assert_eq!(records[2].active, true);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_name = Record::new(2, "".to_string(), 5.0, false);
        assert!(invalid_name.validate().is_err());

        let invalid_value = Record::new(3, "Test".to_string(), -5.0, true);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert!(records.is_empty());

        let stats = calculate_statistics(&records);
        assert_eq!(stats, (0.0, 0.0, 0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .into_iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn transform_column<F>(
        &self,
        records: Vec<Vec<String>>,
        column_index: usize,
        transformer: F,
    ) -> Vec<Vec<String>>
    where
        F: Fn(&str) -> String,
    {
        records
            .into_iter()
            .map(|mut record| {
                if column_index < record.len() {
                    record[column_index] = transformer(&record[column_index]);
                }
                record
            })
            .collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
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
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "25", "New York"]);

        let filtered = processor.filter_records(records, |record| {
            record.get(1).and_then(|age| age.parse::<i32>().ok()) > Some(30)
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);

        let transformed = processor.transform_column(filtered, 2, |city| city.to_uppercase());
        assert_eq!(transformed[0], vec!["Charlie", "35", "PARIS"]);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let avg = calculate_column_average(&records, 0);
        assert!(avg.is_some());
        assert!((avg.unwrap() - 12.666).abs() < 0.001);
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
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_num + 1, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record, line_num + 1)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_num, parts[0]))
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid value format '{}'", line_num, parts[2]))
        })?;

        let active = parts[3].parse::<bool>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid boolean format '{}'", line_num, parts[3]))
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_num)
            ));
        }

        if self.records.iter().any(|r| r.id == record.id) {
            return Err(CsvError::ValidationError(
                format!("Line {}: Duplicate ID {}", line_num, record.id)
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
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
    fn test_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,42.5,true").unwrap();
        writeln!(csv_data, "2,Bob,33.7,false").unwrap();
        writeln!(csv_data, "# This is a comment").unwrap();
        writeln!(csv_data, "").unwrap();
        writeln!(csv_data, "3,Charlie,15.2,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 91.4);
        assert_eq!(processor.filter_active().len(), 2);
    }

    #[test]
    fn test_duplicate_id() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,42.5,true").unwrap();
        writeln!(csv_data, "1,Bob,33.7,false").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_invalid_format() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "not_a_number,Alice,42.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }
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
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse Error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        Self {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;
        
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;
        let mut skip_header = self.has_header;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if skip_header {
                skip_header = false;
                continue;
            }

            if line_content.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        self.validate_records(&records)?;
        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 fields, found {}", parts.len()),
                line_number,
            ));
        }

        let id = parts[0].trim().parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: '{}'", parts[0]),
                line_number,
            )
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(
                "Name cannot be empty".to_string(),
                line_number,
            ));
        }

        let value = parts[2].trim().parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: '{}'", parts[2]),
                line_number,
            )
        })?;

        let active = parts[3].trim().parse::<bool>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid boolean format: '{}'", parts[3]),
                line_number,
            )
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in file".to_string(),
            ));
        }

        let mut seen_ids = std::collections::HashSet::new();
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(CsvError::ValidationError(
                    format!("Duplicate ID found: {}", record.id),
                ));
            }

            if record.value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Negative value found for ID {}: {}", record.id, record.value),
                ));
            }
        }

        Ok(())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_content = "id,name,value,active\n\
                          1,ItemA,10.5,true\n\
                          2,ItemB,20.0,false\n\
                          3,ItemC,15.75,true";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.0);
        assert!(records[2].active);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, mean, std_dev) = CsvProcessor::calculate_statistics(&records);
        
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
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

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
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
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len();
    let average = sum / count as f64;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;
    
    (average, variance.sqrt(), count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item A,25.5,true").unwrap();
        writeln!(temp_file, "2,Item B,30.0,false").unwrap();
        writeln!(temp_file, "3,Item C,42.8,true").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Item A");
        assert_eq!(records[1].active, false);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];
        
        let (avg, std_dev, count) = calculate_statistics(&records);
        assert_eq!(count, 2);
        assert_eq!(avg, 15.0);
        assert!(std_dev - 5.0 < 0.0001);
    }
}