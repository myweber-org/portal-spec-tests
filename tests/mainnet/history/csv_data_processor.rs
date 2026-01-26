
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, name: String, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            category,
            value,
            active,
        }
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category && record.active)
            .collect()
    }

    fn calculate_average(&self, category: Option<&str>) -> f64 {
        let filtered_records: Vec<&DataRecord> = match category {
            Some(cat) => self.records.iter().filter(|r| r.category == cat).collect(),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered_records.iter().map(|r| r.value).sum();
        sum / filtered_records.len() as f64
    }

    fn export_to_csv(&self, file_path: &str, category: Option<&str>) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(file_path)?;

        let records_to_export = match category {
            Some(cat) => self.filter_by_category(cat),
            None => self.records.iter().collect(),
        };

        for record in records_to_export {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    let sample_data = vec![
        DataRecord::new(1, "ItemA".to_string(), "Electronics".to_string(), 299.99, true),
        DataRecord::new(2, "ItemB".to_string(), "Books".to_string(), 24.50, true),
        DataRecord::new(3, "ItemC".to_string(), "Electronics".to_string(), 599.99, false),
        DataRecord::new(4, "ItemD".to_string(), "Clothing".to_string(), 49.99, true),
        DataRecord::new(5, "ItemE".to_string(), "Electronics".to_string(), 199.99, true),
    ];

    processor.records = sample_data;

    let electronics = processor.filter_by_category("Electronics");
    println!("Found {} electronics items", electronics.len());

    let avg_electronics = processor.calculate_average(Some("Electronics"));
    println!("Average electronics value: {:.2}", avg_electronics);

    let max_record = processor.find_max_value();
    if let Some(record) = max_record {
        println!("Highest value item: {} - {}", record.name, record.value);
    }

    processor.export_to_csv("output.csv", Some("Electronics"))?;
    println!("Data exported successfully");

    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
    }
}
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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn read_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|record| record.active)
        .collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;
    
    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_map.entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_map.into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_data(aggregated: &[AggregatedData], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);

    for data in aggregated {
        writer.serialize(data)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_file(input_path)?;
    println!("Total records read: {}", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());
    
    let aggregated_data = aggregate_by_category(&records);
    println!("Categories aggregated: {}", aggregated_data.len());
    
    for data in &aggregated_data {
        println!("Category: {}, Total: {:.2}, Avg: {:.2}, Count: {}",
                 data.category, data.total_value, data.average_value, data.record_count);
    }
    
    write_aggregated_data(&aggregated_data, output_path)?;
    println!("Results written to: {}", output_path);
    
    Ok(())
}

pub fn run_processor() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}