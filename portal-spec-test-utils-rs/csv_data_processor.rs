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

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

pub fn to_uppercase(value: &str) -> String {
    value.to_uppercase()
}

pub fn to_lowercase(value: &str) -> String {
    value.to_lowercase()
}

pub fn is_numeric_record(record: &[String]) -> bool {
    record.iter().all(|field| field.parse::<f64>().is_ok())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records.iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter()
        .map(|record| record.value)
        .sum()
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,75.2,Books").unwrap();
        
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].category, "Books");
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "ItemA".to_string(), value: 100.5, category: "Electronics".to_string() },
            Record { id: 2, name: "ItemB".to_string(), value: 75.2, category: "Books".to_string() },
            Record { id: 3, name: "ItemC".to_string(), value: 50.0, category: "Electronics".to_string() },
        ];

        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Electronics"));
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            Record { id: 1, name: "ItemA".to_string(), value: 100.5, category: "Electronics".to_string() },
            Record { id: 2, name: "ItemB".to_string(), value: 75.2, category: "Books".to_string() },
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 175.7);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = vec![
            Record { id: 1, name: "ItemA".to_string(), value: 100.5, category: "Electronics".to_string() },
            Record { id: 2, name: "ItemB".to_string(), value: 75.2, category: "Books".to_string() },
            Record { id: 3, name: "ItemC".to_string(), value: 150.0, category: "Electronics".to_string() },
        ];

        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 150.0);
    }
}