use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.get(self.filter_column).map(|&val| val == self.filter_value).unwrap_or(false) {
                let transformed_line = parts.iter()
                    .map(|part| part.trim().to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.filter_column > 10 {
            return Err("Filter column index too large".into());
        }
        if self.input_path.is_empty() || self.output_path.is_empty() {
            return Err("File paths cannot be empty".into());
        }
        Ok(())
    }
}

pub fn execute_processing(
    input: &str,
    output: &str,
    column: usize,
    value: &str,
) -> Result<String, Box<dyn Error>> {
    let processor = CsvProcessor::new(input, output, column, value);
    processor.validate()?;
    let count = processor.process()?;
    Ok(format!("Processed {} matching records", count))
}