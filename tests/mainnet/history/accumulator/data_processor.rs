use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    fn process_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(input_path)?;
        let mut writer = Writer::from_writer(File::create(output_path)?);

        for result in reader.deserialize() {
            let record: Record = result?;
            
            if self.filter_record(&record) {
                let processed = self.transform_record(record);
                writer.serialize(processed)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn filter_record(&self, record: &Record) -> bool {
        record.active && record.value >= self.threshold
    }

    fn transform_record(&self, mut record: Record) -> Record {
        record.value = (record.value * 100.0).round() / 100.0;
        record
    }
}

fn validate_data(records: &[Record]) -> bool {
    records.iter().all(|r| !r.name.is_empty() && r.id > 0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = DataProcessor::new(50.0);
    processor.process_file("input.csv", "output.csv")?;
    
    println!("Data processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_record() {
        let processor = DataProcessor::new(50.0);
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 75.5,
            active: true,
        };
        
        assert!(processor.filter_record(&record));
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(0.0);
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 123.456,
            active: true,
        };
        
        let transformed = processor.transform_record(record);
        assert_eq!(transformed.value, 123.46);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_csv_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Test1,10.5,A").unwrap();
        writeln!(file, "2,Test2,20.0,B").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}