use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self, category: Option<&str>) -> f64 {
        let filtered_records: Vec<&Record> = match category {
            Some(cat) => self.filter_by_category(cat),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let total: f64 = filtered_records.iter().map(|r| r.value).sum();
        total / filtered_records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,75.2,Books").unwrap();
        writeln!(temp_file, "3,ItemC,120.8,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg_electronics = processor.calculate_average(Some("Electronics"));
        assert!((avg_electronics - 110.65).abs() < 0.01);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 3);
    }
}use csv::{Reader, Writer};
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

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

fn write_results_to_csv(
    filtered_records: &[&Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(output_path)?;

    wtr.write_record(&["Filtered Records"])?;
    wtr.write_record(&["ID", "Name", "Category", "Value", "Active"])?;

    for record in filtered_records {
        wtr.serialize(record)?;
    }

    wtr.write_record(&[])?;
    wtr.write_record(&["Category Averages"])?;
    wtr.write_record(&["Category", "Average Value"])?;

    for (category, avg) in averages {
        wtr.write_record(&[category, &avg.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);

    println!("Processed {} total records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

    write_results_to_csv(&active_records, &category_averages, output_path)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_results.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}use csv::{Reader, Writer};
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

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut total_value = 0.0;
    let mut active_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;

        if record.value >= min_value {
            writer.serialize(&record)?;

            total_value += record.value;
            if record.active {
                active_count += 1;
            }
        }
    }

    writer.flush()?;

    println!("Processing complete:");
    println!("  Total filtered value: {:.2}", total_value);
    println!("  Active records: {}", active_count);
    println!("  Output written to: {}", output_path);

    Ok(())
}

fn generate_sample_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;

    let sample_data = vec![
        Record { id: 1, name: String::from("Item A"), category: String::from("Electronics"), value: 125.50, active: true },
        Record { id: 2, name: String::from("Item B"), category: String::from("Books"), value: 25.99, active: true },
        Record { id: 3, name: String::from("Item C"), category: String::from("Clothing"), value: 45.75, active: false },
        Record { id: 4, name: String::from("Item D"), category: String::from("Electronics"), value: 89.99, active: true },
        Record { id: 5, name: String::from("Item E"), category: String::from("Home"), value: 15.25, active: true },
    ];

    for record in sample_data {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    println!("Sample CSV generated at: {}", path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "sample_data.csv";
    let output_file = "filtered_data.csv";
    let filter_threshold = 30.0;

    generate_sample_csv(input_file)?;
    process_csv(input_file, output_file, filter_threshold)?;

    Ok(())
}