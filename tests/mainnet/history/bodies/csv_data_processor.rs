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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn get_top_records(&self, limit: usize) -> Vec<&Record> {
        let mut sorted_records: Vec<&Record> = self.records.iter().collect();
        sorted_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted_records.into_iter().take(limit).collect()
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for record in &self.records {
            *counts.entry(record.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,category,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,Electronics,250.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,Books,45.0,false").unwrap();
        writeln!(temp_file, "3,ItemC,Electronics,120.75,true").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);

        let total_value = processor.calculate_total_value();
        assert!((total_value - 416.25).abs() < 0.001);

        let average_value = processor.calculate_average_value();
        assert!((average_value - 138.75).abs() < 0.001);

        let top_records = processor.get_top_records(2);
        assert_eq!(top_records.len(), 2);
        assert_eq!(top_records[0].id, 1);

        let category_counts = processor.count_by_category();
        assert_eq!(category_counts.get("Electronics"), Some(&2));
        assert_eq!(category_counts.get("Books"), Some(&1));

        processor.clear();
        assert_eq!(processor.record_count(), 0);
    }
}