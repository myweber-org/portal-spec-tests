
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