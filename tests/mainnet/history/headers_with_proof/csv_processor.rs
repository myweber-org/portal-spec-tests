use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    file_path: String,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = HashMap::new();
        let mut lines = reader.lines();

        if let Some(header) = lines.next() {
            let headers: Vec<String> = header?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            for line in lines {
                let record = line?;
                let values: Vec<&str> = record.split(',').map(|s| s.trim()).collect();

                if values.len() == headers.len() {
                    for (i, header) in headers.iter().enumerate() {
                        if let Ok(num) = values[i].parse::<f64>() {
                            *results.entry(header.clone()).or_insert(0.0) += num;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn print_summary(&self, data: &HashMap<String, f64>) {
        println!("CSV Data Summary:");
        for (key, value) in data {
            println!("  {}: {:.2}", key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Product,Price,Quantity").unwrap();
        writeln!(temp_file, "Apple,1.50,10").unwrap();
        writeln!(temp_file, "Banana,0.75,20").unwrap();
        writeln!(temp_file, "Orange,2.00,15").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();

        assert_eq!(result.get("Product"), None);
        assert_eq!(result.get("Price"), Some(&4.25));
        assert_eq!(result.get("Quantity"), Some(&45.0));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

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

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 || self.should_include_row(&parts) {
                writeln!(output_file, "{}", line)?;
            }
        }

        Ok(())
    }

    fn should_include_row(&self, row: &[&str]) -> bool {
        if self.filter_column < row.len() {
            row[self.filter_column] == self.filter_value
        } else {
            false
        }
    }
}

pub fn transform_data(input: &str) -> String {
    input
        .split(',')
        .map(|field| field.trim().to_uppercase())
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_transform_data() {
        let input = "hello, world, rust";
        let expected = "HELLO|WORLD|RUST";
        assert_eq!(transform_data(input), expected);
    }

    #[test]
    fn test_csv_processor() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        let content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";

        fs::write(test_input, content).unwrap();

        let processor = CsvProcessor::new(test_input, test_output, 2, "active");
        let result = processor.process();

        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,status\n1,alice,active\n3,charlie,active\n";
        assert_eq!(output_content, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

            if parts.get(self.filter_column).map(|&v| v == self.filter_value).unwrap_or(false) {
                writeln!(output_file, "{}", line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column<F>(&self, transform_fn: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<&str> = line.split(',').collect();
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() > 1 {
                parts[1] = &transform_fn(parts[1]);
                let transformed_line = parts.join(",");
                writeln!(output_file, "{}", transformed_line)?;
            }
        }

        Ok(())
    }
}

pub fn validate_csv_path(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    if path_obj.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        return Err("File must have .csv extension".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,value\n1,test,100\n2,other,200\n3,test,300\n";
        fs::write(test_input, content).unwrap();

        let processor = CsvProcessor::new(test_input, test_output, 1, "test");
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("1,test,100"));
        assert!(output_content.contains("3,test,300"));
        assert!(!output_content.contains("2,other,200"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}