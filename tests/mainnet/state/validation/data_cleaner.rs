use csv::{Reader, Writer};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct RawRecord {
    id: String,
    value: String,
    category: String,
}

#[derive(Debug)]
struct CleanRecord {
    id: u32,
    value: f64,
    category: String,
}

impl CleanRecord {
    fn from_raw(raw: RawRecord) -> Result<Self, Box<dyn Error>> {
        let id = raw.id.parse::<u32>()?;
        let value = raw.value.parse::<f64>()?;
        let category = raw.category.trim().to_lowercase();

        if value < 0.0 {
            return Err("Negative values not allowed".into());
        }

        Ok(CleanRecord {
            id,
            value,
            category,
        })
    }
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    writer.write_record(&["id", "value", "category"])?;

    for result in reader.deserialize() {
        let raw: RawRecord = result?;
        
        match CleanRecord::from_raw(raw) {
            Ok(clean) => {
                writer.write_record(&[
                    clean.id.to_string(),
                    clean.value.to_string(),
                    clean.category,
                ])?;
            }
            Err(e) => eprintln!("Skipping record: {}", e),
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input.csv", "output.csv")?;
    println!("Data cleaning completed successfully");
    Ok(())
}