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

    pub fn clean_file(&self, input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut cleaned_count = 0;
        let mut lines_iter = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, header_result)) = lines_iter.next() {
                let header = header_result?;
                writeln!(output_file, "{}", header)?;
            }
        }

        for (line_num, line_result) in lines_iter {
            let line = line_result?;
            let cleaned_line = self.clean_line(&line);

            if !cleaned_line.is_empty() {
                writeln!(output_file, "{}", cleaned_line)?;
                cleaned_count += 1;
            } else {
                eprintln!("Warning: Empty line at position {}", line_num + 1);
            }
        }

        Ok(cleaned_count)
    }

    fn clean_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        let cleaned_parts: Vec<String> = parts
            .iter()
            .map(|&field| field.trim().to_string())
            .filter(|field| !field.is_empty())
            .collect();

        cleaned_parts.join(&self.delimiter.to_string())
    }

    pub fn validate_row(&self, row: &str) -> bool {
        let parts: Vec<&str> = row.split(self.delimiter).collect();
        !parts.is_empty() && parts.iter().all(|&field| !field.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_line() {
        let processor = CsvProcessor::new(',', false);
        let dirty_line = "  apple, banana ,, cherry  ";
        let cleaned = processor.clean_line(dirty_line);
        assert_eq!(cleaned, "apple,banana,cherry");
    }

    #[test]
    fn test_validate_row() {
        let processor = CsvProcessor::new(',', false);
        assert!(processor.validate_row("a,b,c"));
        assert!(!processor.validate_row("a,,c"));
        assert!(!processor.validate_row(""));
    }

    #[test]
    fn test_clean_file() -> Result<(), Box<dyn Error>> {
        let mut temp_input = NamedTempFile::new()?;
        writeln!(temp_input, "fruit,color,weight")?;
        writeln!(temp_input, " apple , red , 100 ")?;
        writeln!(temp_input, "banana,yellow,150")?;
        writeln!(temp_input, ",,")?;

        let mut temp_output = NamedTempFile::new()?;
        let processor = CsvProcessor::new(',', true);

        let cleaned = processor.clean_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;

        assert_eq!(cleaned, 2);

        let output_content = std::fs::read_to_string(temp_output.path())?;
        assert!(output_content.contains("fruit,color,weight"));
        assert!(output_content.contains("apple,red,100"));
        assert!(output_content.contains("banana,yellow,150"));
        assert!(!output_content.contains(",,"));

        Ok(())
    }
}