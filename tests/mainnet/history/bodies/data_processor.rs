use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn remove_record_by_id(&mut self, id: u32) -> bool {
        let initial_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() < initial_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(1, "Test1".to_string(), 10.5, "A".to_string());
        processor.add_record(2, "Test2".to_string(), 20.0, "B".to_string());
        processor.add_record(3, "Test3".to_string(), 15.0, "A".to_string());
        
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.calculate_average(), 15.166666666666666);
        
        assert!(processor.remove_record_by_id(2));
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        
        let input_path = temp_input.path().to_str().unwrap();
        let output_path = temp_output.path().to_str().unwrap();
        
        let mut wtr = Writer::from_path(input_path)?;
        wtr.serialize(Record {
            id: 1,
            name: "Item1".to_string(),
            value: 100.0,
            category: "Electronics".to_string(),
        })?;
        wtr.serialize(Record {
            id: 2,
            name: "Item2".to_string(),
            value: 50.0,
            category: "Books".to_string(),
        })?;
        wtr.flush()?;
        
        processor.load_from_csv(input_path)?;
        assert_eq!(processor.records.len(), 2);
        
        processor.save_filtered_to_csv("Electronics", output_path)?;
        
        Ok(())
    }
}