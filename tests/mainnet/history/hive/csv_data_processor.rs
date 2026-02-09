use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            self.records
                .iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        })
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(values.iter().sum()),
            "avg" => Some(values.iter().sum::<f64>() / values.len() as f64),
            "min" => values.iter().copied().reduce(f64::min),
            "max" => values.iter().copied().reduce(f64::max),
            _ => None,
        }
    }

    pub fn write_filtered_to_file(&self, filtered_data: &[Vec<String>], output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;
        
        writeln!(file, "{}", self.headers.join(","))?;
        
        for record in filtered_data {
            writeln!(file, "{}", record.join(","))?;
        }
        
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub fn process_csv_sample() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::from_file("data/sample.csv")?;
    
    println!("Loaded {} records", processor.get_record_count());
    println!("Headers: {:?}", processor.get_headers());
    
    let filtered = processor.filter_by_column("status", |value| value == "active");
    println!("Active records: {}", filtered.len());
    
    if let Some(total) = processor.aggregate_numeric_column("value", "sum") {
        println!("Total value: {:.2}", total);
    }
    
    processor.write_filtered_to_file(&filtered, "data/active_records.csv")?;
    
    Ok(())
}