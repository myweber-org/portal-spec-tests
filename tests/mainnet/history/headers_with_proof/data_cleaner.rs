use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

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
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_lines(input: &str) -> Vec<String> {
        let lines: Vec<String> = input.lines().map(|s| s.trim().to_string()).collect();
        let unique_lines: HashSet<String> = lines.into_iter().collect();
        let mut sorted_lines: Vec<String> = unique_lines.into_iter().collect();
        sorted_lines.sort();
        sorted_lines
    }

    pub fn process_stdin() -> io::Result<()> {
        let stdin = io::stdin();
        let mut buffer = String::new();
        
        for line in stdin.lock().lines() {
            buffer.push_str(&line?);
            buffer.push('\n');
        }
        
        let cleaned = Self::clean_lines(&buffer);
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        
        for line in cleaned {
            writeln!(handle, "{}", line)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_lines() {
        let input = "zebra\nalpha\nalpha\nbeta\nzebra";
        let result = DataCleaner::clean_lines(input);
        assert_eq!(result, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = DataCleaner::clean_lines(input);
        assert!(result.is_empty());
    }
}