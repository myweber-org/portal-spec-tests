
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
}