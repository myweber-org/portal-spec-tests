
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0 && !category.is_empty();
        DataRecord {
            id,
            timestamp,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("Invalid ID");
        }
        if self.timestamp < 0 {
            return Err("Invalid timestamp");
        }
        if self.value < 0.0 || self.value > 1000.0 {
            return Err("Value out of range");
        }
        if self.category.is_empty() {
            return Err("Empty category");
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_count: 0,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        self.records.push(record);
        self.processed_count += 1;
        Ok(())
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut loaded = 0;
        for (line_num, line) in content.lines().enumerate().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let timestamp = parts[1].parse::<i64>().unwrap_or(0);
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].trim();

            let record = DataRecord::new(id, timestamp, value, category);
            if let Err(e) = self.add_record(record) {
                eprintln!("Error loading record at line {}: {}", line_num + 1, e);
            } else {
                loaded += 1;
            }
        }

        Ok(loaded)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.filter_valid();
        let count = valid_records.len();

        if count == 0 {
            return Statistics::default();
        }

        let values: Vec<f64> = valid_records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;

        Statistics {
            total_records: self.records.len(),
            valid_records: count,
            min_value: min,
            max_value: max,
            average_value: avg,
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.processed_count = 0;
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub average_value: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics: Total={}, Valid={}, Min={:.2}, Max={:.2}, Avg={:.2}",
            self.total_records,
            self.valid_records,
            self.min_value,
            self.max_value,
            self.average_value
        )
    }
}