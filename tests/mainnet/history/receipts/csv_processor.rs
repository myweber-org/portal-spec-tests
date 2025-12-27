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
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
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

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }
        
        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative: {}", line_num, record.value)
            ));
        }
        
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|record| record.id == id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _header = lines.next().transpose()?;
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

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .into_iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn extract_column(&self, records: Vec<Vec<String>>, column_index: usize) -> Vec<String> {
        records
            .into_iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "Carol,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            vec!["A".to_string(), "10".to_string()],
            vec!["B".to_string(), "20".to_string()],
            vec!["C".to_string(), "30".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |record| {
            record.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0) > 15
        });

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "B");
        assert_eq!(filtered[1][0], "C");
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["X".to_string(), "100".to_string(), "Active".to_string()],
            vec!["Y".to_string(), "200".to_string(), "Inactive".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        let column = processor.extract_column(records, 1);

        assert_eq!(column, vec!["100", "200"]);
    }
}