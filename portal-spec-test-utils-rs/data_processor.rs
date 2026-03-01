
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let valid = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.filter_valid();
        
        if valid_records.is_empty() {
            return Statistics::default();
        }

        let values: Vec<f64> = valid_records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();

        Statistics {
            count,
            min,
            max,
            mean,
            std_dev,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics: count={}, min={:.2}, max={:.2}, mean={:.2}, std_dev={:.2}",
            self.count, self.min, self.max, self.mean, self.std_dev
        )
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,false").unwrap();
        writeln!(temp_file, "3,15.7,category_a,true").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average().unwrap();
        assert!((average - 13.1).abs() < 0.01);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.min, 10.5);
        assert_eq!(stats.max, 15.7);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("category_a").unwrap().len(), 2);
        assert_eq!(groups.get("category_b").unwrap().len(), 1);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
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

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
}

fn process_sample_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "ItemA".to_string(), 42.5, "Alpha".to_string());
    processor.add_record(2, "ItemB".to_string(), 33.2, "Beta".to_string());
    processor.add_record(3, "ItemC".to_string(), 67.8, "Alpha".to_string());
    processor.add_record(4, "ItemD".to_string(), 19.1, "Gamma".to_string());
    processor.add_record(5, "ItemE".to_string(), 55.3, "Beta".to_string());
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    let alpha_records = processor.filter_by_category("Alpha");
    println!("Alpha category records: {}", alpha_records.len());
    
    processor.sort_by_value();
    println!("Sorted records:");
    for record in &processor.records {
        println!("  {}: {} = {}", record.id, record.name, record.value);
    }
    
    Ok(())
}