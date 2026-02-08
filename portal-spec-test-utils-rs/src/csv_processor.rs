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
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    pub fn transform_column(&self, transform_column: usize, transform_fn: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut processed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let mut columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > transform_column {
                let original_value = columns[transform_column];
                let transformed_value = transform_fn(original_value);
                columns[transform_column] = &transformed_value;
                
                let new_line = columns.join(",");
                writeln!(output_file, "{}", new_line)?;
                processed_count += 1;
            }
        }
        
        Ok(processed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    fn uppercase_transform(value: &str) -> String {
        value.to_uppercase()
    }
    
    #[test]
    fn test_csv_processing() {
        let test_input = "test_data.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        fs::write(test_input, content).unwrap();
        
        let processor = CsvProcessor::new(test_input, test_output, 2, "active");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("alice"));
        assert!(output_content.contains("charlie"));
        assert!(!output_content.contains("bob"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}