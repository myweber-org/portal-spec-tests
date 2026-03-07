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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err("CSV file contains no valid records".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_count = records.iter().filter(|r| r.active).count();
    
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let average = sum / records.len() as f64;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / records.len() as f64;
    
    (average, variance.sqrt(), active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Product A,25.50,true").unwrap();
        writeln!(temp_file, "2,Product B,30.75,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Product C,42.00,yes").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Product A");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].active, true);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];

        let (avg, std_dev, active_count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert!(std_dev - 8.164965 < 0.0001);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_invalid_csv_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid,Product,25.50,true").unwrap();
        
        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            skip_header: false,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let parsed_row = self.parse_line(&line);
            let filtered_row = self.filter_columns(parsed_row);
            results.push(filtered_row);
        }

        Ok(results)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        line.split(self.config.delimiter)
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn filter_columns(&self, row: Vec<String>) -> Vec<String> {
        if self.config.selected_columns.is_empty() {
            return row;
        }

        self.config
            .selected_columns
            .iter()
            .filter_map(|&idx| row.get(idx).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_column_selection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "A,B,C,D").unwrap();
        writeln!(temp_file, "1,2,3,4").unwrap();

        let config = CsvConfig {
            selected_columns: vec![0, 2],
            ..Default::default()
        };
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result[0], vec!["1", "3"]);
    }

    #[test]
    fn test_skip_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header").unwrap();
        writeln!(temp_file, "data").unwrap();

        let config = CsvConfig {
            skip_header: true,
            ..Default::default()
        };
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["data"]);
    }
}