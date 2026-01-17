
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _headers = lines.next().transpose()?;
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_columns: usize) -> bool {
        record.len() == expected_columns && 
        !record.iter().any(|field| field.is_empty())
    }

    pub fn transform_numeric_fields(records: &[Vec<String>]) -> Vec<Vec<String>> {
        records
            .iter()
            .map(|record| {
                record
                    .iter()
                    .map(|field| {
                        if let Ok(num) = field.parse::<f64>() {
                            format!("{:.2}", num)
                        } else {
                            field.clone()
                        }
                    })
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30.5,London").unwrap();
        writeln!(temp_file, "Bob,,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!(processor.validate_record(&result[0], 3));
        assert!(!processor.validate_record(&result[2], 3));
        
        let transformed = CsvProcessor::transform_numeric_fields(&result);
        assert_eq!(transformed[1][1], "30.50");
    }
}