use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

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
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut writer = Writer::from_path(file_path)?;

        for record in filtered {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn remove_inactive(&mut self) {
        self.records.retain(|record| record.active);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    processor.add_record(DataRecord::new(1, "A", 100.5, true));
    processor.add_record(DataRecord::new(2, "B", 200.3, false));
    processor.add_record(DataRecord::new(3, "A", 150.7, true));
    processor.add_record(DataRecord::new(4, "C", 300.9, true));

    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }

    let category_a_records = processor.filter_by_category("A");
    println!("Category A records: {}", category_a_records.len());

    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());

    processor.remove_inactive();
    println!("Records after removing inactive: {}", processor.records.len());

    processor.save_filtered_to_csv("filtered_data.csv", "A")?;
    println!("Filtered data saved to filtered_data.csv");

    Ok(())
}