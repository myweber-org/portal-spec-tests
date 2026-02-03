use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();
    
    println!("Enter data (press Ctrl+D when finished):");
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            input.push_str(&line);
            input.push('\n');
        }
    }
    
    let cleaned = clean_data(&input);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")
        .expect("Failed to create output file");
    
    output_file.write_all(cleaned.as_bytes())
        .expect("Failed to write to output file");
    
    println!("Data cleaned and saved to cleaned_output.txt");
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut wtr = WriterBuilder::new().from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}