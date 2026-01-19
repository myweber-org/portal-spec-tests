use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(Self { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| record.get(col_index).map_or(false, |v| v == value))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let mut sum = 0.0;
            let mut count = 0;

            for record in &self.records {
                if let Some(value_str) = record.get(col_index) {
                    if let Ok(value) = value_str.parse::<f64>() {
                        sum += value;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                Ok(sum)
            } else {
                Err("No valid numeric values found".into())
            }
        } else {
            Err(format!("Column '{}' not found", column_name).into())
        }
    }

    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

pub fn process_csv_file(input_path: &str, filter_column: Option<&str>, filter_value: Option<&str>, aggregate_column: Option<&str>) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(input_path)?;

    println!("Loaded CSV with {} columns and {} records", processor.headers.len(), processor.records.len());
    println!("Columns: {:?}", processor.get_column_names());

    if let (Some(col), Some(val)) = (filter_column, filter_value) {
        let filtered = processor.filter_by_column(col, val);
        println!("Filtered records ({} = {}): {}", col, val, filtered.len());
        for record in filtered.iter().take(5) {
            println!("  {:?}", record);
        }
    }

    if let Some(col) = aggregate_column {
        match processor.aggregate_numeric_column(col) {
            Ok(sum) => println!("Sum of column '{}': {:.2}", col, sum),
            Err(e) => println!("Could not aggregate column '{}': {}", col, e),
        }
    }

    Ok(())
}