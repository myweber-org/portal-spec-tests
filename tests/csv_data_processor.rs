use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_and_aggregate<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    target_category: &str,
    min_value: f64,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut total_value = 0.0;
    let mut record_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.category == target_category && record.value >= min_value && record.active {
            writer.serialize(&record)?;
            total_value += record.value;
            record_count += 1;
        }
    }

    writer.flush()?;

    if record_count > 0 {
        let average_value = total_value / record_count as f64;
        println!("Processed {} records", record_count);
        println!("Total value: {:.2}", total_value);
        println!("Average value: {:.2}", average_value);
    } else {
        println!("No records matched the criteria");
    }

    Ok(())
}

fn generate_sample_data<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;

    let sample_records = vec![
        Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 249.99, active: true },
        Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 19.99, active: true },
        Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 599.99, active: false },
        Record { id: 4, name: "Item D".to_string(), category: "Electronics".to_string(), value: 129.99, active: true },
        Record { id: 5, name: "Item E".to_string(), category: "Clothing".to_string(), value: 49.99, active: true },
    ];

    for record in sample_records {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "sample_data.csv";
    let output_file = "filtered_data.csv";
    
    generate_sample_data(input_file)?;
    
    filter_and_aggregate(
        input_file,
        output_file,
        "Electronics",
        100.0
    )?;

    Ok(())
}