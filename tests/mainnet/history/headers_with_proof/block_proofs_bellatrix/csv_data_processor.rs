use csv::{Reader, Writer};
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

    let mut total_records = 0;
    let mut filtered_records = 0;
    let mut sum_values = 0.0;

    for result in reader.deserialize() {
        let record: Record = result?;
        total_records += 1;

        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
            filtered_records += 1;
            sum_values += record.value;
        }
    }

    writer.flush()?;

    if filtered_records > 0 {
        let average_value = sum_values / filtered_records as f64;
        println!("Processed {} records", total_records);
        println!("Filtered {} records with value >= {}", filtered_records, min_value);
        println!("Average value of filtered records: {:.2}", average_value);
    } else {
        println!("No records matched the filter criteria");
    }

    Ok(())
}

fn aggregate_by_category(input_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut category_totals = std::collections::HashMap::new();
    let mut category_counts = std::collections::HashMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            let total = category_totals.entry(record.category.clone()).or_insert(0.0);
            *total += record.value;
            
            let count = category_counts.entry(record.category).or_insert(0);
            *count += 1;
        }
    }

    println!("Aggregation by category:");
    for (category, total) in category_totals {
        if let Some(count) = category_counts.get(&category) {
            let average = total / *count as f64;
            println!("Category: {}, Total: {:.2}, Count: {}, Average: {:.2}", 
                    category, total, count, average);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered_output.csv";
    
    match process_csv(input_file, output_file, 50.0) {
        Ok(_) => {
            println!("CSV processing completed successfully");
            aggregate_by_category(output_file)?;
        }
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}