
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<CsvRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn save_filtered_results<P: AsRef<Path>>(
        &self,
        category: &str,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut file = File::create(output_path)?;
        
        writeln!(file, "id,name,category,value,active")?;
        for record in filtered {
            writeln!(
                file,
                "{},{},{},{},{}",
                record.id, record.name, record.category, record.value, record.active
            )?;
        }
        
        Ok(())
    }

    pub fn get_statistics(&self) -> String {
        let total = self.calculate_total_value();
        let average = self.calculate_average_value();
        let active_count = self.filter_active().len();
        let total_count = self.records.len();
        
        format!(
            "Total records: {}\nActive records: {}\nTotal value: {:.2}\nAverage value: {:.2}",
            total_count, active_count, total, average
        )
    }
}

pub fn process_csv_data(input_file: &str, output_file: &str, category_filter: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(input_file)?;
    
    let stats = processor.get_statistics();
    println!("Processing statistics:\n{}", stats);
    
    processor.save_filtered_results(category_filter, output_file)?;
    println!("Filtered data saved to: {}", output_file);
    
    Ok(())
}