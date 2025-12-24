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

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
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

fn save_processed_data(
    records: &[Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.write_record(&[category, &avg.to_string()])?;
    }

    writer.write_record(&[])?;
    writer.write_record(&["ID", "Name", "Category", "Value", "Active"])?;
    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    let active_records = filter_active_records(all_records);
    let category_averages = calculate_category_averages(&active_records);

    save_processed_data(&active_records, &category_averages, output_path)?;

    println!("Processed {} active records", active_records.len());
    println!("Found {} categories", category_averages.len());

    for (category, avg) in &category_averages {
        println!("Category '{}': average value = {:.2}", category, avg);
    }

    Ok(())
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
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "B".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let filtered = filter_active_records(records);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_category_averages() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "A".to_string(),
                value: 20.0,
                active: true,
            },
        ];

        let averages = calculate_category_averages(&records);
        assert_eq!(averages.len(), 1);
        assert_eq!(averages[0].0, "A");
        assert_eq!(averages[0].1, 15.0);
    }
}