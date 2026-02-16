use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

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

    fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let active_records: Vec<&Record> = self
            .records
            .iter()
            .filter(|record| record.active)
            .collect();

        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        for record in active_records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, category: String, value: f64, active: bool) {
        let record = Record {
            id,
            name,
            category,
            value,
            active,
        };
        self.records.push(record);
    }

    fn remove_record(&mut self, id: u32) -> Option<Record> {
        if let Some(pos) = self.records.iter().position(|r| r.id == id) {
            Some(self.records.remove(pos))
        } else {
            None
        }
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "Item A".to_string(), "Electronics".to_string(), 299.99, true);
    processor.add_record(2, "Item B".to_string(), "Books".to_string(), 24.50, true);
    processor.add_record(3, "Item C".to_string(), "Electronics".to_string(), 599.99, false);
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Found {} electronics items", electronics.len());
    
    let avg_value = processor.calculate_average();
    println!("Average value: {:.2}", avg_value);
    
    processor.export_active_records("active_records.csv")?;
    
    if let Some(removed) = processor.remove_record(2) {
        println!("Removed record: {:?}", removed);
    }
    
    Ok(())
}