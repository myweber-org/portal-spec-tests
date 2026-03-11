
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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
        let columns: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if columns.is_empty() {
            return Err(format!("Empty line at line {}", line_num).into());
        }

        Ok(CsvRecord { columns })
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records found".into());
        }

        let expected_columns = records[0].columns.len();
        for (idx, record) in records.iter().enumerate() {
            if record.columns.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    idx + 1,
                    record.columns.len(),
                    expected_columns
                )
                .into());
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
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let parser = CsvParser::new(',', true);
        let records = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["John", "30", "New York"]);
        assert_eq!(records[1].columns, vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            CsvRecord {
                columns: vec!["a".to_string(), "b".to_string()],
            },
            CsvRecord {
                columns: vec!["c".to_string(), "d".to_string()],
            },
        ];

        let parser = CsvParser::new(',', false);
        assert!(parser.validate_records(&records).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let records = vec![
            CsvRecord {
                columns: vec!["a".to_string(), "b".to_string()],
            },
            CsvRecord {
                columns: vec!["c".to_string()],
            },
        ];

        let parser = CsvParser::new(',', false);
        assert!(parser.validate_records(&records).is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();
            
            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .enumerate()
            .filter(|(line_num, line)| {
                !line.trim().is_empty() && !(self.has_header && *line_num == 0)
            })
            .map(|(_, line)| {
                line.split(self.delimiter)
                    .map(|field| field.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_string_with_header() {
        let parser = CsvParser::new(',', true);
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_string_without_header() {
        let parser = CsvParser::new(';', false);
        let data = "John;30;New York\nJane;25;London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_file() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value")?;
        writeln!(temp_file, "1,apple,5.5")?;
        writeln!(temp_file, "2,banana,3.2")?;
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path())?;
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "apple", "5.5"]);
        
        Ok(())
    }

    #[test]
    fn test_empty_lines_skipped() {
        let parser = CsvParser::new(',', false);
        let data = "a,b,c\n\n\nd,e,f\n\n";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["d", "e", "f"]);
    }
}use std::error::Error;
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
        content
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                if line_num == 0 && self.has_header {
                    None
                } else {
                    let record: Vec<String> = line
                        .split(self.delimiter)
                        .map(|field| field.trim().to_string())
                        .collect();
                    if record.is_empty() {
                        None
                    } else {
                        Some(record)
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let parser = CsvParser::new().has_header(false);
        let data = "name,age,city\nAlice,30,New York\nBob,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';').has_header(false);
        let data = "name;age;city\nAlice;30;New York";
        let result = parser.parse_string(data);
        
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_skip_header() {
        let parser = CsvParser::new().has_header(true);
        let data = "name,age,city\nAlice,30,New York\nBob,25,London";
        let result = parser.parse_string(data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }
}