use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty record at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid records found in file".into());
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut records = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty record at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid records found in content".into());
        }

        Ok(records)
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_csv_parsing() {
        let content = "name,age,city\nJohn,30,New York\nJane,25,London";
        let parser = CsvParser::new();
        let result = parser.parse_string(content).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let content = "name|age|city\nJohn|30|New York";
        let parser = CsvParser::new().delimiter('|');
        let result = parser.parse_string(content).unwrap();
        
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_no_header() {
        let content = "John,30,New York\nJane,25,London";
        let parser = CsvParser::new().has_header(false);
        let result = parser.parse_string(content).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let parser = CsvParser::new();
        let result = parser.parse_string(content);
        
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
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

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line_content = line?;
            let record = self.parse_line(&line_content, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, Box<dyn Error>> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err(format!("Line {} is empty", line_number).into());
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

        for (i, field) in record.fields.iter().enumerate() {
            if field.is_empty() {
                return Err(format!("Field {} is empty", i + 1));
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
    fn test_parse_csv_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::new(',', true);
        let records = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].fields, vec!["Alice", "30", "New York"]);
        assert_eq!(records[1].fields, vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let parser = CsvParser::new(',', false);
        let valid_record = CsvRecord {
            fields: vec!["test".to_string(), "data".to_string()],
        };
        let invalid_record = CsvRecord {
            fields: vec!["".to_string(), "data".to_string()],
        };

        assert!(parser.validate_record(&valid_record, 2).is_ok());
        assert!(parser.validate_record(&invalid_record, 2).is_err());
    }
}