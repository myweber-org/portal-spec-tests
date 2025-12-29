
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
}