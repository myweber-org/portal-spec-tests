use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| CsvError::IoError(e.to_string()))?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(CsvError::ParseError(
                format!("Line {}: expected 3 columns, found {}", line_num + 1, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid ID format '{}'", line_num + 1, parts[0])
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: name cannot be empty", line_num + 1)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Line {}: invalid value format '{}'", line_num + 1, parts[2])
            ))?;

        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: value cannot be negative", line_num + 1)
            ));
        }

        records.push(CsvRecord { id, name, value });
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Record {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }

    fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv(input_path: &str, output_path: &str, category_filter: Option<&str>) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for result in csv_reader.deserialize() {
        let mut record: Record = result?;

        if !record.is_valid() {
            continue;
        }

        if let Some(filter) = category_filter {
            if record.category != filter {
                continue;
            }
        }

        record.transform_value(1.1);
        csv_writer.serialize(&record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn generate_sample_data() -> Vec<Record> {
    vec![
        Record::new(1, "ItemA", "Electronics", 99.99, true),
        Record::new(2, "ItemB", "Books", 24.50, true),
        Record::new(3, "ItemC", "Electronics", 149.99, false),
        Record::new(4, "", "Books", 15.00, true),
        Record::new(5, "ItemE", "Clothing", -10.00, true),
    ]
}

fn validate_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.is_valid())
        .collect()
}

fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter()
        .filter(|r| r.active)
        .map(|r| r.value)
        .sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_records = generate_sample_data();
    let valid_records = validate_records(&sample_records);
    
    println!("Total valid records: {}", valid_records.len());
    println!("Total value of active records: {:.2}", calculate_total_value(&sample_records));

    process_csv("input.csv", "output.csv", Some("Electronics"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test", "Category", 100.0, true);
        assert!(valid_record.is_valid());

        let invalid_name = Record::new(2, "", "Category", 100.0, true);
        assert!(!invalid_name.is_valid());

        let invalid_value = Record::new(3, "Test", "Category", -50.0, true);
        assert!(!invalid_value.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "Test", "Category", 100.0, true);
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_total_calculation() {
        let records = vec![
            Record::new(1, "A", "Cat1", 10.0, true),
            Record::new(2, "B", "Cat2", 20.0, false),
            Record::new(3, "C", "Cat1", 30.0, true),
        ];
        assert_eq!(calculate_total_value(&records), 40.0);
    }
}