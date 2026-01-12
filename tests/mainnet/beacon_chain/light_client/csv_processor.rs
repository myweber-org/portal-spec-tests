
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if columns.is_empty() {
                continue;
            }

            records.push(CsvRecord { columns });
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.columns.get(column_index).cloned())
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

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord {
                columns: vec!["A".to_string(), "10".to_string()],
            },
            CsvRecord {
                columns: vec!["B".to_string(), "20".to_string()],
            },
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |record| {
            record.columns.get(1).map_or(false, |age| age == "10")
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].columns[0], "A");
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
    pub expected_columns: Option<usize>,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
            expected_columns: None,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            line_count += 1;

            if index == 0 && self.config.has_headers {
                continue;
            }

            let columns: Vec<&str> = line.split(self.config.delimiter).collect();
            
            if let Some(expected) = self.config.expected_columns {
                if columns.len() != expected {
                    return Err(format!(
                        "Line {} has {} columns, expected {}",
                        line_count,
                        columns.len(),
                        expected
                    ).into());
                }
            }

            if column_count.is_none() {
                column_count = Some(columns.len());
            } else if column_count != Some(columns.len()) {
                return Err(format!(
                    "Inconsistent column count at line {}",
                    line_count
                ).into());
            }
        }

        if line_count == 0 {
            return Err("File is empty".into());
        }

        if self.config.has_headers && line_count == 1 {
            return Err("File contains only headers".into());
        }

        Ok(line_count - if self.config.has_headers { 1 } else { 0 })
    }

    pub fn extract_column<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut result = Vec::new();

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;

            if index == 0 && self.config.has_headers {
                continue;
            }

            let columns: Vec<&str> = line.split(self.config.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!(
                    "Column index {} out of bounds at line {}",
                    column_index,
                    index + 1
                ).into());
            }

            result.push(columns[column_index].to_string());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_validate_valid_csv() {
        let csv_content = "name,age,city\nJohn,30,NYC\nJane,25,LA\n";
        let file = create_test_csv(csv_content);
        
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let result = processor.validate_file(file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_validate_inconsistent_columns() {
        let csv_content = "name,age,city\nJohn,30\nJane,25,LA,extra\n";
        let file = create_test_csv(csv_content);
        
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let result = processor.validate_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_column() {
        let csv_content = "name,age,city\nJohn,30,NYC\nJane,25,LA\n";
        let file = create_test_csv(csv_content);
        
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let result = processor.extract_column(file.path(), 0);
        assert!(result.is_ok());
        let names = result.unwrap();
        assert_eq!(names, vec!["John", "Jane"]);
    }
}