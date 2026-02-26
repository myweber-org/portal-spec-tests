use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let mut seen = HashSet::new();
    for result in csv_reader.records() {
        let record = result?;
        let row_string = record.iter().collect::<String>();
        
        if seen.insert(row_string) {
            csv_writer.write_record(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub struct DataCleaner {
    missing_value_replacement: String,
}

impl DataCleaner {
    pub fn new(replacement: &str) -> Self {
        DataCleaner {
            missing_value_replacement: replacement.to_string(),
        }
    }

    pub fn clean_csv(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(input_file);

        let output_file = File::create(output_path)?;
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(output_file);

        let headers = reader.headers()?.clone();
        writer.write_record(&headers)?;

        for result in reader.records() {
            let record = result?;
            let cleaned_record: Vec<String> = record
                .iter()
                .map(|field| {
                    if field.trim().is_empty() || field == "NULL" || field == "null" {
                        self.missing_value_replacement.clone()
                    } else {
                        field.to_string()
                    }
                })
                .collect();

            writer.write_record(&cleaned_record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn count_missing_values(&self, path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut missing_count = 0;

        for result in reader.records() {
            let record = result?;
            for field in record.iter() {
                if field.trim().is_empty() || field == "NULL" || field == "null" {
                    missing_count += 1;
                }
            }
        }

        Ok(missing_count)
    }
}