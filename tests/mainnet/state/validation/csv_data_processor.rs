use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }
            
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 4 {
                let record = CsvRecord {
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,18.75,Electronics").unwrap();
        
        let filepath = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(filepath).unwrap();
        
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.filter_by_category("Electronics").len(), 2);
        
        let avg = processor.calculate_average_value();
        assert!((avg - 28.75).abs() < 0.01);
        
        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 42.0);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active)
        .collect()
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

fn save_processed_data<P: AsRef<Path>>(
    records: &[&Record],
    averages: &[(String, f64)],
    output_path: P
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;
    
    writer.write_record(&["ID", "Name", "Category", "Value", "Active"])?;
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.write_record(&[])?;
    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.write_record(&[category, &avg.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);
    
    save_processed_data(&active_records, &category_averages, output_path)?;
    
    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
    Ok(())
}