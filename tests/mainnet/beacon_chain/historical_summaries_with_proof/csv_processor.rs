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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if index == 0 && self.has_header {
                continue;
            }

            if let Some(expected) = column_count {
                if columns.len() != expected {
                    return Err(format!("Line {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected).into());
                }
            } else {
                column_count = Some(columns.len());
            }

            for (col_idx, value) in columns.iter().enumerate() {
                if value.trim().is_empty() {
                    return Err(format!("Empty value at line {}, column {}", 
                        index + 1, col_idx + 1).into());
                }
            }

            line_count += 1;
        }

        if line_count == 0 {
            return Err("File contains no data rows".into());
        }

        Ok(line_count)
    }

    pub fn extract_column<P: AsRef<Path>>(&self, file_path: P, column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut result = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!("Column index {} out of bounds on line {}", 
                    column_index, index + 1).into());
            }

            result.push(columns[column_index].trim().to_string());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let column_data = processor.extract_column(temp_file.path(), 0).unwrap();
        assert_eq!(column_data, vec!["Alice", "Bob"]);
    }
}