use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
}

impl Record {
    fn from_csv_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return None;
        }

        let id = parts[0].parse().ok()?;
        let name = parts[1].to_string();
        let category = parts[2].to_string();
        let value = parts[3].parse().ok()?;

        Some(Record {
            id,
            name,
            category,
            value,
        })
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

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(record) = Record::from_csv_line(&line) {
                self.records.push(record);
            }
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self, category: &str) -> Option<f64> {
        let filtered = self.filter_by_category(category);
        if filtered.is_empty() {
            return None;
        }

        let sum: f64 = filtered.iter().map(|record| record.value).sum();
        Some(sum / filtered.len() as f64)
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn summary(&self) {
        println!("Total records: {}", self.records.len());

        let categories: Vec<String> = self
            .records
            .iter()
            .map(|r| r.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for category in categories {
            if let Some(avg) = self.calculate_average(&category) {
                println!("Category '{}': average = {:.2}", category, avg);
            }
        }

        if let Some(max_record) = self.find_max_value() {
            println!(
                "Maximum value: {} (ID: {}, Category: {})",
                max_record.value, max_record.id, max_record.category
            );
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_file("data.csv") {
        Ok(_) => {
            processor.summary();
            
            let tech_records = processor.filter_by_category("Technology");
            println!("\nTechnology records: {}", tech_records.len());
            
            for record in tech_records.iter().take(3) {
                println!("  - {}: ${:.2}", record.name, record.value);
            }
        }
        Err(e) => eprintln!("Error loading file: {}", e),
    }

    Ok(())
}