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

            if let Some(value) = parts.get(self.filter_column) {
                if value.trim() == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column(&self, column_index: usize, transformer: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut transformed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<&str> = line.split(',').collect();
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(value) = parts.get_mut(column_index) {
                let transformed = transformer(value);
                parts[column_index] = &transformed;
                transformed_count += 1;
            }

            let new_line = parts.join(",");
            writeln!(output_file, "{}", new_line)?;
        }

        Ok(transformed_count)
    }
}

pub fn uppercase_transformer(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transformer(value: &str) -> String {
    value.trim().to_string()
}