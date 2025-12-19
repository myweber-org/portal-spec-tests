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