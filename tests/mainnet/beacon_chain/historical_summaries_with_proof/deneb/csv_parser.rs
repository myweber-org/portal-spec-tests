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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
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
            .skip(if self.has_header { 1 } else { 0 })
            .map(|line| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .filter(|record: &Vec<String>| !record.is_empty())
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
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,status").unwrap();
        writeln!(temp_file, "1,100,active").unwrap();
        writeln!(temp_file, "2,200,inactive").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "100", "active"]);
    }

    #[test]
    fn test_empty_fields() {
        let parser = CsvParser::new(',', false);
        let data = "field1,,field3\n,field2,";
        let result = parser.parse_string(data);
        assert_eq!(result[0], vec!["field1", "", "field3"]);
        assert_eq!(result[1], vec!["", "field2", ""]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvParser {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvParser {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let fields: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.len() != headers.len() {
                return Err(format!(
                    "Row has {} fields, expected {}", 
                    fields.len(), 
                    headers.len()
                ).into());
            }
            
            records.push(fields);
        }

        Ok(CsvParser { headers, records })
    }

    pub fn get_column<T: FromStr>(&self, column_name: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T::Err: Error + 'static,
    {
        let index = self.headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut result = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let value = record[index].parse::<T>()?;
            result.push(value);
        }

        Ok(result)
    }

    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    pub fn get_record(&self, row_index: usize) -> Option<&[String]> {
        self.records.get(row_index).map(|v| v.as_slice())
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(parser.column_count(), 3);
        assert_eq!(parser.row_count(), 2);
        
        let ages: Vec<u32> = parser.get_column("age").unwrap();
        assert_eq!(ages, vec![30, 25]);
        
        let record = parser.get_record(0).unwrap();
        assert_eq!(record, &["Alice", "30", "New York"]);
    }

    #[test]
    fn test_missing_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();

        let parser = CsvParser::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let result: Result<Vec<u32>, _> = parser.get_column("salary");
        assert!(result.is_err());
    }
}