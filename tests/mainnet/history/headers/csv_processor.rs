
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvConfig {
    input_path: String,
    output_path: Option<String>,
    selected_columns: Vec<usize>,
    delimiter: char,
    has_headers: bool,
}

impl CsvConfig {
    pub fn new(input_path: String) -> Self {
        CsvConfig {
            input_path,
            output_path: None,
            selected_columns: Vec::new(),
            delimiter: ',',
            has_headers: true,
        }
    }

    pub fn with_output_path(mut self, path: String) -> Self {
        self.output_path = Some(path);
        self
    }

    pub fn with_selected_columns(mut self, columns: Vec<usize>) -> Self {
        self.selected_columns = columns;
        self
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }
}

pub fn process_csv(config: CsvConfig) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(&config.input_path)?;
    let reader = BufReader::new(input_file);
    
    let output: Box<dyn Write> = match &config.output_path {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };
    let mut writer = io::BufWriter::new(output);

    let mut lines = reader.lines();
    
    if config.has_headers {
        if let Some(header_line) = lines.next() {
            let headers: Vec<String> = header_line?
                .split(config.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if config.selected_columns.is_empty() {
                writeln!(writer, "{}", headers.join(&config.delimiter.to_string()))?;
            } else {
                let selected_headers: Vec<String> = config.selected_columns
                    .iter()
                    .filter_map(|&idx| headers.get(idx).cloned())
                    .collect();
                writeln!(writer, "{}", selected_headers.join(&config.delimiter.to_string()))?;
            }
        }
    }

    for line_result in lines {
        let line = line_result?;
        let fields: Vec<&str> = line.split(config.delimiter).collect();
        
        let selected_fields = if config.selected_columns.is_empty() {
            fields.iter().map(|&s| s).collect()
        } else {
            config.selected_columns
                .iter()
                .filter_map(|&idx| fields.get(idx))
                .copied()
                .collect()
        };
        
        writeln!(writer, "{}", selected_fields.join(&config.delimiter.to_string()))?;
    }

    writer.flush()?;
    Ok(())
}

pub fn validate_csv_file(path: &str) -> Result<bool, Box<dyn Error>> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Ok(false);
    }
    
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let first_line = reader.lines().next();
    match first_line {
        Some(Ok(line)) => Ok(!line.trim().is_empty()),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let input_content = "name,age,city\nAlice,30,London\nBob,25,Paris";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let config = CsvConfig::new(input_file.path().to_str().unwrap().to_string())
            .with_output_path(output_file.path().to_str().unwrap().to_string())
            .with_selected_columns(vec![0, 2])
            .with_headers(true);
        
        assert!(process_csv(config).is_ok());
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output_content, "name,city\nAlice,London\nBob,Paris\n");
    }

    #[test]
    fn test_file_validation() {
        let valid_file = NamedTempFile::new().unwrap();
        fs::write(valid_file.path(), "test,data\n").unwrap();
        
        let result = validate_csv_file(valid_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let invalid_result = validate_csv_file("non_existent_file.csv");
        assert!(invalid_result.is_ok());
        assert!(!invalid_result.unwrap());
    }
}