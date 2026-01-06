use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    pub headers: Vec<String>,
    pub records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            self.headers = first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn validate_records(&self) -> bool {
        for record in &self.records {
            if record.len() != self.headers.len() {
                return false;
            }
        }
        true
    }

    pub fn transform_column(&mut self, column_index: usize, transform_fn: fn(&str) -> String) {
        for record in &mut self.records {
            if column_index < record.len() {
                record[column_index] = transform_fn(&record[column_index]);
            }
        }
    }

    pub fn filter_records(&self, predicate: fn(&[String]) -> bool) -> Vec<Vec<String>> {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_column_stats(&self, column_index: usize) -> Option<(f64, f64, f64)> {
        if column_index >= self.headers.len() {
            return None;
        }

        let mut values = Vec::new();
        for record in &self.records {
            if column_index < record.len() {
                if let Ok(num) = record[column_index].parse::<f64>() {
                    values.push(num);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn numeric_filter(record: &[String]) -> bool {
    record.iter().all(|field| field.parse::<f64>().is_ok())
}