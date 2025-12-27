
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn with_filter(mut self, column: usize, value: &str) -> Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut processed_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let fields: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let should_include = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < fields.len() => fields[col] == val,
                _ => true,
            };

            if should_include {
                writeln!(output_file, "{}", line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }
}

pub fn transform_data(input: &str) -> String {
    input
        .trim()
        .split(',')
        .map(|field| field.to_uppercase())
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_transform_data() {
        let input = "hello,world,rust";
        let expected = "HELLO|WORLD|RUST";
        assert_eq!(transform_data(input), expected);
    }

    #[test]
    fn test_csv_processor() -> Result<(), Box<dyn Error>> {
        let test_input = "test_data.csv";
        let test_output = "test_output.csv";

        let content = "id,name,value\n1,apple,100\n2,banana,200\n3,apple,150\n";
        fs::write(test_input, content)?;

        let processor = CsvProcessor::new(test_input, test_output)
            .with_filter(1, "apple");

        let processed = processor.process()?;
        assert_eq!(processed, 2);

        let output_content = fs::read_to_string(test_output)?;
        assert!(output_content.contains("1,apple,100"));
        assert!(output_content.contains("3,apple,150"));
        assert!(!output_content.contains("banana"));

        fs::remove_file(test_input)?;
        fs::remove_file(test_output)?;

        Ok(())
    }
}