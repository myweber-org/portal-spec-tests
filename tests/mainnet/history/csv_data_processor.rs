
use std::error::Error;
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

    pub fn read_and_validate<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                continue;
            }

            if self.has_header && line_number == 1 {
                continue;
            }

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.iter().any(|field| field.is_empty())
    }

    pub fn transform_numeric_fields(
        &self,
        records: Vec<Vec<String>>,
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::new();

        for record in records {
            if column_index >= record.len() {
                return Err("Column index out of bounds".into());
            }

            let mut new_record = record.clone();
            if let Ok(value) = new_record[column_index].parse::<f64>() {
                let transformed_value = value * multiplier;
                new_record[column_index] = transformed_value.to_string();
            }

            transformed.push(new_record);
        }

        Ok(transformed)
    }

    pub fn filter_records(
        &self,
        records: Vec<Vec<String>>,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Vec<Vec<String>> {
        records.into_iter().filter(|r| predicate(r)).collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
    }
}