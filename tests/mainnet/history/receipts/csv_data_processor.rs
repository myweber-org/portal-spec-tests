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

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
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

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut aggregates = HashMap::new();
    
    for record in records {
        let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }

    aggregates.into_iter().collect()
}

fn write_results<P: AsRef<Path>>(path: P, results: &[(String, f64)]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);

    for (category, total) in results {
        wtr.serialize((category, total))?;
    }

    wtr.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&active_records);
    
    write_results(output_path, &aggregated_data)?;
    
    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated {} category aggregates", aggregated_data.len());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}