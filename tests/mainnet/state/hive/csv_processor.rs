
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

    pub fn set_filter(&mut self, column: usize, value: &str) -> &mut Self {
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
            let columns: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let should_include = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < columns.len() => columns[col] == val,
                _ => true,
            };

            if should_include {
                let transformed_line = self.transform_row(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_row(&self, columns: &[&str]) -> String {
        let transformed: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(idx, &col)| {
                if idx == 0 {
                    col.to_uppercase()
                } else {
                    col.replace('"', "'")
                }
            })
            .collect();
        transformed.join(",")
    }
}

pub fn validate_csv_format(path: &str) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line_result in reader.lines().take(5) {
        let line = line_result?;
        if line.split(',').count() < 2 {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let input_data = "name,age,city\njohn,25,new york\njane,30,london\njack,25,paris";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let mut processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.set_filter(1, "25");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("JOHN"));
        assert!(output_content.contains("JACK"));
        assert!(!output_content.contains("JANE"));
    }

    #[test]
    fn test_validation() {
        let valid_data = "field1,field2,field3\nvalue1,value2,value3";
        let invalid_data = "single_field\nanother_field";
        
        let mut valid_file = NamedTempFile::new().unwrap();
        write!(valid_file, "{}", valid_data).unwrap();
        
        let mut invalid_file = NamedTempFile::new().unwrap();
        write!(invalid_file, "{}", invalid_data).unwrap();
        
        assert!(validate_csv_format(valid_file.path().to_str().unwrap()).unwrap());
        assert!(!validate_csv_format(invalid_file.path().to_str().unwrap()).unwrap());
    }
}