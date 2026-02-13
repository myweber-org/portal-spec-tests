use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(value) = parts.get(self.filter_column) {
                if value.trim() == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column(&self, column_index: usize, transformer: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut transformed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<&str> = line.split(',').collect();
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(value) = parts.get_mut(column_index) {
                let transformed = transformer(value);
                parts[column_index] = &transformed;
                transformed_count += 1;
            }

            let new_line = parts.join(",");
            writeln!(output_file, "{}", new_line)?;
        }

        Ok(transformed_count)
    }
}

pub fn uppercase_transformer(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transformer(value: &str) -> String {
    value.trim().to_string()
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            category: parts[3].to_string(),
        })
    }
}

fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", index + 1, e),
        }
    }

    Ok(records)
}

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;

    let mut aggregation: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = aggregation
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    aggregation
        .into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Processed {} records", records.len());
    
    let aggregation = aggregate_by_category(&records);
    
    println!("\nAggregation by category:");
    for (category, total, count) in aggregation {
        println!("{}: ${:.2} ({} items)", category, total, count);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_parsing() {
        let line = "1,ProductA,29.99,Electronics";
        let record = Record::from_csv_line(line).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "ProductA");
        assert_eq!(record.value, 29.99);
        assert_eq!(record.category, "Electronics");
    }

    #[test]
    fn test_invalid_record() {
        let line = "1,ProductA";
        let result = Record::from_csv_line(line);
        assert!(result.is_err());
    }
}