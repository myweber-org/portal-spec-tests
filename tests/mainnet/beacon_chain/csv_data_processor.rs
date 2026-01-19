
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

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn transform_column(
        &self,
        records: &mut [Vec<String>],
        column_index: usize,
        transformer: impl Fn(&str) -> String,
    ) -> Result<(), String> {
        for record in records.iter_mut() {
            if column_index < record.len() {
                let original = &record[column_index];
                record[column_index] = transformer(original);
            } else {
                return Err(format!("Column index {} out of bounds", column_index));
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
            vec!["Charlie".to_string(), "35".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(&records, |record| {
            record[1].parse::<i32>().unwrap_or(0) > 30
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
    }

    #[test]
    fn test_transform_column() {
        let mut records = vec![
            vec!["alice".to_string(), "30".to_string()],
            vec!["bob".to_string(), "25".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        processor
            .transform_column(&mut records, 0, |s| s.to_uppercase())
            .unwrap();

        assert_eq!(records[0][0], "ALICE");
        assert_eq!(records[1][0], "BOB");
    }
}