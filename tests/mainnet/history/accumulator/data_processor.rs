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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold && record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_value(threshold);
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn count_active(&self) -> usize {
        self.records.iter().filter(|r| r.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = "id,name,value,active\n\
                        1,ItemA,25.5,true\n\
                        2,ItemB,15.0,false\n\
                        3,ItemC,30.0,true";
        
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_data).unwrap();
        
        assert!(processor.load_from_csv(temp_file.path().to_str().unwrap()).is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.count_active(), 2);
        
        let filtered = processor.filter_by_value(20.0);
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 23.5).abs() < 0.001);
    }
}