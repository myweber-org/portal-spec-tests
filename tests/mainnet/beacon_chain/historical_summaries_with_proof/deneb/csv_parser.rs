use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
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

            let record = self.parse_line(&line)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str) -> Result<CsvRecord, Box<dyn Error>> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err("Empty line".into());
        }

        Ok(CsvRecord { fields })
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records found".into());
        }

        let expected_len = records[0].fields.len();
        for (idx, record) in records.iter().enumerate() {
            if record.fields.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    idx,
                    record.fields.len(),
                    expected_len
                ).into());
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
    fn test_parse_simple_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let parser = CsvParser::new(',', true);
        let records = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].fields, vec!["John", "30", "New York"]);
        assert_eq!(records[1].fields, vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            CsvRecord { fields: vec!["a".to_string(), "b".to_string()] },
            CsvRecord { fields: vec!["c".to_string(), "d".to_string()] },
        ];
        
        let parser = CsvParser::new(',', false);
        assert!(parser.validate_records(&records).is_ok());
    }

    #[test]
    fn test_validate_inconsistent_records() {
        let records = vec![
            CsvRecord { fields: vec!["a".to_string(), "b".to_string()] },
            CsvRecord { fields: vec!["c".to_string()] },
        ];
        
        let parser = CsvParser::new(',', false);
        assert!(parser.validate_records(&records).is_err());
    }
}