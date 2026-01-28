use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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

    pub fn clean_file<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut cleaned_count = 0;
        let mut line_number = 0;

        for line in reader.lines() {
            let original_line = line?;
            line_number += 1;

            if line_number == 1 && self.has_header {
                writeln!(output_file, "{}", original_line)?;
                continue;
            }

            let cleaned_line = self.clean_line(&original_line);
            if cleaned_line.is_some() {
                writeln!(output_file, "{}", cleaned_line.unwrap())?;
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    fn clean_line(&self, line: &str) -> Option<String> {
        let fields: Vec<&str> = line.split(self.delimiter).collect();
        
        if fields.len() < 2 {
            return None;
        }

        let cleaned_fields: Vec<String> = fields
            .iter()
            .map(|field| field.trim().to_string())
            .filter(|field| !field.is_empty())
            .collect();

        if cleaned_fields.len() == fields.len() {
            Some(cleaned_fields.join(&self.delimiter.to_string()))
        } else {
            None
        }
    }

    pub fn validate_row(&self, row: &str) -> bool {
        let fields: Vec<&str> = row.split(self.delimiter).collect();
        fields.len() >= 2 && fields.iter().all(|f| !f.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_line_valid() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("apple,banana,cherry");
        assert_eq!(result, Some("apple,banana,cherry".to_string()));
    }

    #[test]
    fn test_clean_line_with_spaces() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("  apple , banana , cherry  ");
        assert_eq!(result, Some("apple,banana,cherry".to_string()));
    }

    #[test]
    fn test_clean_line_invalid() {
        let processor = CsvProcessor::new(',', false);
        let result = processor.clean_line("apple,,cherry");
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_row() {
        let processor = CsvProcessor::new('|', false);
        assert!(processor.validate_row("field1|field2|field3"));
        assert!(!processor.validate_row("field1||field3"));
        assert!(!processor.validate_row("single"));
    }

    #[test]
    fn test_clean_file_integration() {
        let input_content = "name,age,city\nJohn,25,NYC\nJane,,London\nBob,30,\nAlice,35,Paris";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.clean_file(input_file.path(), output_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert_eq!(output_content, "name,age,city\nJohn,25,NYC\nAlice,35,Paris\n");
    }
}