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
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn export_to_csv(&self, file_path: &str, records: &[&Record]) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    fn remove_record(&mut self, id: u32) -> bool {
        let initial_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() < initial_len
    }
}

fn generate_sample_data() -> Vec<Record> {
    vec![
        Record::new(1, "Item A", "Electronics", 299.99, true),
        Record::new(2, "Item B", "Books", 24.99, true),
        Record::new(3, "Item C", "Electronics", 599.99, false),
        Record::new(4, "Item D", "Clothing", 49.99, true),
        Record::new(5, "Item E", "Books", 14.99, true),
        Record::new(6, "Item F", "Electronics", 199.99, true),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    let sample_data = generate_sample_data();
    for record in sample_data {
        processor.add_record(record);
    }

    println!("Total records: {}", processor.records.len());
    println!("Total value: ${:.2}", processor.calculate_total_value());
    println!("Average value: ${:.2}", processor.calculate_average_value());

    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics items: {}", electronics.len());

    let active_items = processor.filter_active();
    println!("Active items: {}", active_items.len());

    if let Some(max_record) = processor.find_max_value() {
        println!("Most expensive item: {} (${})", max_record.name, max_record.value);
    }

    processor.export_to_csv("output.csv", &active_items)?;
    println!("Exported {} active items to output.csv", active_items.len());

    let removed = processor.remove_record(3);
    println!("Record 3 removed: {}", removed);
    println!("Remaining records: {}", processor.records.len());

    Ok(())
}