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

    pub fn clean_file<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<usize, Box<dyn Error>> {
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
            let cleaned_line = self.clean_line(&line, line_num + 1);

            if let Some(valid_line) = cleaned_line {
                writeln!(output_file, "{}", valid_line)?;
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    fn clean_line(&self, line: &str, line_number: usize) -> Option<String> {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            eprintln!("Warning: Empty line at {}", line_number);
            return None;
        }

        let fields: Vec<&str> = trimmed.split(self.delimiter).collect();
        
        if fields.iter().any(|field| field.trim().is_empty()) {
            eprintln!("Warning: Missing field at line {}", line_number);
            return None;
        }

        let cleaned_fields: Vec<String> = fields
            .iter()
            .map(|field| field.trim().to_string())
            .collect();

        Some(cleaned_fields.join(&self.delimiter.to_string()))
    }
}

pub fn validate_csv_format<P: AsRef<Path>>(path: P, delimiter: char) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Ok(false),
    };

    let field_count = first_line.split(delimiter).count();

    for line_result in lines {
        let line = line_result?;
        if line.split(delimiter).count() != field_count {
            return Ok(false);
        }
    }

    Ok(true)
}