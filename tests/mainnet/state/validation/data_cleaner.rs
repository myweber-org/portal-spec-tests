use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_lowercase())
            .collect();

        wtr.write_record(&cleaned_record)?;
    }

    wtr.flush()?;
    Ok(())
}