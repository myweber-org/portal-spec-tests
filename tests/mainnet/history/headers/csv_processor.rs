use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub fn validate_and_transform(
        &self,
        input_path: &str,
        output_path: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;
        let mut processed_rows = 0;

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line.is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if self.has_header && line_number == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let transformed_fields: Vec<String> = fields
                .iter()
                .map(|field| {
                    let trimmed = field.trim();
                    if trimmed.is_empty() {
                        "NULL".to_string()
                    } else if let Ok(num) = trimmed.parse::<f64>() {
                        format!("{:.2}", num)
                    } else {
                        trimmed.to_uppercase()
                    }
                })
                .collect();

            let transformed_line = transformed_fields.join(&self.delimiter.to_string());
            writeln!(output_file, "{}", transformed_line)?;
            processed_rows += 1;
        }

        Ok(processed_rows)
    }

    pub fn count_records(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let input_data = "name,age,city\nJohn,25,New York\nJane,30.5,London\n,42,Paris";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_data.as_bytes()).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_and_transform(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        let expected = "name,age,city\nJOHN,25.00,NEW YORK\nJANE,30.50,LONDON\nNULL,42.00,PARIS\n";
        assert_eq!(output_content, expected);
    }

    #[test]
    fn test_record_count() {
        let csv_data = "id,value\n1,test\n2,data\n3,example";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_data.as_bytes()).unwrap();

        let processor = CsvProcessor::new(',', true);
        let count = processor.count_records(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(count, 3);
    }
}use std::error::Error;
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

            if parts.get(self.filter_column) == Some(&self.filter_value.as_str()) {
                let transformed_line = self.transform_record(&parts);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_record(&self, record: &[&str]) -> String {
        record
            .iter()
            .map(|field| field.trim().to_uppercase())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_data.csv";
        let test_output = "test_output.csv";
        let content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        
        fs::write(test_input, content).unwrap();
        
        let processor = CsvProcessor::new(test_input, test_output, 2, "active");
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("ALICE"));
        assert!(!output_content.contains("BOB"));
        assert!(output_content.contains("CHARLIE"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}