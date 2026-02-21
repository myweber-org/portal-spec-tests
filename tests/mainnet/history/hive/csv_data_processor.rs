use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

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
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn export_filtered(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    let sample_data = vec![
        Record::new(1, "ItemA", "Electronics", 249.99, true),
        Record::new(2, "ItemB", "Books", 19.99, true),
        Record::new(3, "ItemC", "Electronics", 599.99, false),
        Record::new(4, "ItemD", "Clothing", 49.99, true),
        Record::new(5, "ItemE", "Electronics", 129.99, true),
    ];
    
    processor.records = sample_data;
    
    println!("Average value: {:.2}", processor.calculate_average());
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Active Electronics items: {}", electronics.len());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Highest value item: {} - ${:.2}", max_record.name, max_record.value);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Processing error: {}", e);
    }
}