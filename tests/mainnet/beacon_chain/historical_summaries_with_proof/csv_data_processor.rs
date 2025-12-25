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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn transform_numeric_fields(&self, records: &mut Vec<Vec<String>>, column_index: usize) {
        for record in records.iter_mut() {
            if column_index < record.len() {
                if let Ok(num) = record[column_index].parse::<f64>() {
                    let transformed = (num * 100.0).round() / 100.0;
                    record[column_index] = format!("{:.2}", transformed);
                }
            }
        }
    }

    pub fn validate_record_lengths(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", i + 1, record.len(), expected_len));
            }
        }
        Ok(())
    }
}

pub fn filter_records_by_column(
    records: Vec<Vec<String>>,
    column_index: usize,
    predicate: impl Fn(&str) -> bool,
) -> Vec<Vec<String>> {
    records
        .into_iter()
        .filter(|record| {
            column_index < record.len() && predicate(&record[column_index])
        })
        .collect()
}