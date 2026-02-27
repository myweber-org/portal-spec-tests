
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub struct CsvParser {
    pub delimiter: char,
    pub has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: false,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, Box<dyn Error>> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut inside_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '"' {
                if inside_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 1;
                } else {
                    inside_quotes = !inside_quotes;
                }
            } else if ch == self.delimiter && !inside_quotes {
                fields.push(current_field.clone());
                current_field.clear();
            } else {
                current_field.push(ch);
            }

            i += 1;
        }

        fields.push(current_field);

        if inside_quotes {
            return Err(format!("Unclosed quotes on line {}", line_num).into());
        }

        Ok(CsvRecord { fields })
    }

    pub fn validate_record(&self, record: &CsvRecord, expected_fields: usize) -> Result<(), String> {
        if record.fields.len() != expected_fields {
            return Err(format!(
                "Expected {} fields, found {}",
                expected_fields,
                record.fields.len()
            ));
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
    fn test_basic_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::new().with_header(true);
        let records = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].fields, vec!["Alice", "30", "New York"]);
        assert_eq!(records[1].fields, vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_quoted_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "data").unwrap();
        writeln!(temp_file, "\"value,with,commas\",normal").unwrap();

        let parser = CsvParser::new();
        let records = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(records[0].fields, vec!["value,with,commas", "normal"]);
    }

    #[test]
    fn test_validation() {
        let record = CsvRecord {
            fields: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let parser = CsvParser::new();

        assert!(parser.validate_record(&record, 3).is_ok());
        assert!(parser.validate_record(&record, 2).is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_headers: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_headers: true,
        }
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_headers {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for line in content.lines() {
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        records
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

    #[test]
    fn test_parse_string() {
        let parser = CsvParser::new();
        let data = "name,age,city\nAlice,30,New York\nBob,25,London";
        let records = parser.parse_string(data);
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
        assert_eq!(records[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let data = "name;age;city\nAlice;30;New York";
        let records = parser.parse_string(data);
        
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["12.0".to_string(), "25.0".to_string()],
        ];
        
        let avg = calculate_column_average(&records, 0);
        assert_eq!(avg, Some(12.666666666666666));
        
        let avg_none = calculate_column_average(&records, 2);
        assert_eq!(avg_none, None);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }
}

#[derive(Debug)]
enum ParseError {
    IoError(String),
    ParseIntError(String),
    ParseFloatError(String),
    ParseBoolError(String),
    InvalidColumnCount(usize, usize),
    EmptyFile,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::ParseIntError(msg) => write!(f, "Parse integer error: {}", msg),
            ParseError::ParseFloatError(msg) => write!(f, "Parse float error: {}", msg),
            ParseError::ParseBoolError(msg) => write!(f, "Parse boolean error: {}", msg),
            ParseError::InvalidColumnCount(expected, found) => {
                write!(f, "Expected {} columns, found {}", expected, found)
            }
            ParseError::EmptyFile => write!(f, "File is empty"),
        }
    }
}

impl Error for ParseError {}

fn parse_csv(file_path: &str) -> Result<Vec<Record>, ParseError> {
    let file = File::open(file_path)
        .map_err(|e| ParseError::IoError(e.to_string()))?;
    
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_count = 0;

    for line in reader.lines() {
        line_count += 1;
        let line = line.map_err(|e| ParseError::IoError(e.to_string()))?;
        
        if line.trim().is_empty() {
            continue;
        }

        let columns: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if columns.len() != 4 {
            return Err(ParseError::InvalidColumnCount(4, columns.len()));
        }

        let id = u32::from_str(columns[0])
            .map_err(|_| ParseError::ParseIntError(format!("Line {}: '{}'", line_count, columns[0])))?;
        
        let name = columns[1].to_string();
        
        let value = f64::from_str(columns[2])
            .map_err(|_| ParseError::ParseFloatError(format!("Line {}: '{}'", line_count, columns[2])))?;
        
        let active = match columns[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(ParseError::ParseBoolError(format!("Line {}: '{}'", line_count, columns[3]))),
        };

        records.push(Record::new(id, name, value, active));
    }

    if records.is_empty() {
        return Err(ParseError::EmptyFile);
    }

    Ok(records)
}

fn validate_records(records: &[Record]) -> Vec<String> {
    let mut warnings = Vec::new();
    
    for record in records {
        if record.name.is_empty() {
            warnings.push(format!("Record ID {} has empty name", record.id));
        }
        
        if record.value < 0.0 {
            warnings.push(format!("Record ID {} has negative value: {}", record.id, record.value));
        }
        
        if record.value > 1000.0 {
            warnings.push(format!("Record ID {} has unusually high value: {}", record.id, record.value));
        }
    }
    
    warnings
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    
    (mean, variance, std_dev)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    
    match parse_csv(file_path) {
        Ok(records) => {
            println!("Successfully parsed {} records", records.len());
            
            let warnings = validate_records(&records);
            if !warnings.is_empty() {
                println!("Validation warnings:");
                for warning in warnings {
                    println!("  - {}", warning);
                }
            }
            
            let active_records: Vec<&Record> = records.iter()
                .filter(|r| r.active)
                .collect();
            println!("Active records: {}", active_records.len());
            
            let (mean, variance, std_dev) = calculate_statistics(&records);
            println!("Statistics:");
            println!("  Mean: {:.2}", mean);
            println!("  Variance: {:.2}", variance);
            println!("  Standard Deviation: {:.2}", std_dev);
            
            let max_record = records.iter()
                .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
            
            if let Some(record) = max_record {
                println!("Record with maximum value:");
                println!("  ID: {}, Name: {}, Value: {}", record.id, record.name, record.value);
            }
            
            Ok(())
        }
        Err(e) => {
            eprintln!("Error parsing CSV: {}", e);
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,200.0,false").unwrap();
        
        let records = parse_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[0].value, 100.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_parse_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,100.5").unwrap();
        
        let result = parse_csv(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(ParseError::InvalidColumnCount(4, 3))));
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            Record::new(1, String::new(), -10.0, true),
            Record::new(2, "Bob".to_string(), 1500.0, false),
        ];
        
        let warnings = validate_records(&records);
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, true),
            Record::new(3, "C".to_string(), 30.0, true),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}