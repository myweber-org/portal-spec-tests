use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        Self {
            id,
            category: category.to_string(),
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self { records: Vec::new() }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let id: u32 = record[0].parse()?;
            let category = record[1].to_string();
            let value: f64 = record[2].parse()?;
            let active: bool = record[3].parse()?;
            
            self.records.push(DataRecord::new(id, &category, value, active));
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self, category: Option<&str>) -> f64 {
        let filtered_records: Vec<&DataRecord> = match category {
            Some(cat) => self.filter_by_category(cat),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered_records.iter().map(|r| r.value).sum();
        sum / filtered_records.len() as f64
    }

    fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().from_writer(file);
        
        wtr.write_record(&["id", "category", "value", "active"])?;
        
        for record in self.records.iter().filter(|r| r.active) {
            wtr.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.active.to_string(),
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average(None);
        
        (min, max, avg)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    processor.add_record(DataRecord::new(101, "electronics", 299.99, true));
    processor.add_record(DataRecord::new(102, "books", 19.99, false));
    
    let electronics_avg = processor.calculate_average(Some("electronics"));
    println!("Electronics average value: {:.2}", electronics_avg);
    
    let (min, max, avg) = processor.get_statistics();
    println!("Statistics - Min: {:.2}, Max: {:.2}, Avg: {:.2}", min, max, avg);
    
    processor.export_active_records("active_records.csv")?;
    
    let filtered = processor.filter_by_category("books");
    println!("Found {} book records", filtered.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "test", 100.0, true));
        processor.add_record(DataRecord::new(2, "test", 200.0, true));
        
        assert_eq!(processor.calculate_average(Some("test")), 150.0);
        assert_eq!(processor.filter_by_category("test").len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "a", 10.0, true));
        processor.add_record(DataRecord::new(2, "b", 20.0, true));
        processor.add_record(DataRecord::new(3, "c", 30.0, true));
        
        let (min, max, avg) = processor.get_statistics();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }
}