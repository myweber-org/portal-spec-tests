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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn read_csv_data(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_data(
    aggregated: &[AggregatedData],
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for data in aggregated {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    let records = read_csv_data(input_path)?;
    println!("Total records read: {}", records.len());

    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());

    let aggregated_data = aggregate_by_category(&records);
    println!("Categories aggregated: {}", aggregated_data.len());

    for data in &aggregated_data {
        println!(
            "Category: {}, Total: {:.2}, Average: {:.2}, Count: {}",
            data.category, data.total_value, data.average_value, data.record_count
        );
    }

    write_aggregated_data(&aggregated_data, output_path)?;
    println!("Results written to: {}", output_path.display());

    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    if let Err(e) = process_csv_data(input_file, output_file) {
        eprintln!("Error processing CSV data: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 50.0,
                active: false,
            },
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 300.0);
        assert_eq!(aggregated[0].average_value, 150.0);
        assert_eq!(aggregated[0].record_count, 2);
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;

        let test_data = "id,name,category,value,active\n1,Test Item,Test Category,42.5,true\n";
        std::fs::write(temp_input.path(), test_data)?;

        process_csv_data(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;

        assert!(temp_output.path().exists());
        Ok(())
    }
}