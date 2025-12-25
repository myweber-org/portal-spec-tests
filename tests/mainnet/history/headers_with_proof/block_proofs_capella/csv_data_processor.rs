use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
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

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
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

    fn aggregate_by_category(&self) -> HashMap<String, (f64, usize)> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            if !record.active {
                continue;
            }

            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let active_values: Vec<f64> = self
            .records
            .iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .collect();

        if active_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = active_values.iter().sum();
        let count = active_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = active_values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data.csv")?;

    let filtered = processor.filter_by_category("electronics");
    println!("Filtered records: {}", filtered.len());

    let aggregates = processor.aggregate_by_category();
    for (category, (total, count)) in aggregates {
        println!("Category: {}, Total: {:.2}, Count: {}", category, total, count);
    }

    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
             mean, variance, std_dev);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = Record::from_csv_line("1,electronics,100.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "electronics");
        assert_eq!(record.value, 100.5);
        assert!(record.active);
    }

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,active").unwrap();
        writeln!(file, "1,electronics,100.5,true").unwrap();
        writeln!(file, "2,clothing,50.0,true").unwrap();
        writeln!(file, "3,electronics,75.0,false").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_file(file.path().to_str().unwrap()).unwrap();

        let filtered = processor.filter_by_category("electronics");
        assert_eq!(filtered.len(), 1);

        let aggregates = processor.aggregate_by_category();
        assert!(aggregates.contains_key("electronics"));
        assert!(aggregates.contains_key("clothing"));
    }
}