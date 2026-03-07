use std::error::Error;
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

    pub fn validate_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let columns: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if let Some(expected_count) = column_count {
                if columns.len() != expected_count {
                    return Err(format!(
                        "Line {} has {} columns, expected {}",
                        index + 1,
                        columns.len(),
                        expected_count
                    ).into());
                }
            } else {
                column_count = Some(columns.len());
            }

            line_count += 1;
        }

        if line_count == 0 {
            return Err("File is empty or contains only whitespace".into());
        }

        Ok(line_count)
    }

    pub fn transform_column<F>(&self, file_path: &str, column_index: usize, transform_fn: F) -> Result<Vec<String>, Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut lines_iter = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, header_line)) = lines_iter.next() {
                let _ = header_line?;
            }
        }

        for (line_num, line) in lines_iter {
            let line_content = line?;
            let columns: Vec<&str> = line_content.split(self.delimiter).collect();

            if column_index >= columns.len() {
                return Err(format!(
                    "Column index {} out of bounds on line {}",
                    column_index,
                    line_num + 1
                ).into());
            }

            let transformed = transform_fn(columns[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }

    pub fn filter_rows<P>(&self, file_path: &str, predicate: P) -> Result<Vec<String>, Box<dyn Error>>
    where
        P: Fn(&[&str]) -> bool,
    {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut filtered_rows = Vec::new();
        let mut lines_iter = reader.lines();

        if self.has_header {
            if let Some(header_line) = lines_iter.next() {
                filtered_rows.push(header_line?);
            }
        }

        for line in lines_iter {
            let line_content = line?;
            let columns: Vec<&str> = line_content.split(self.delimiter).collect();

            if predicate(&columns) {
                filtered_rows.push(line_content);
            }
        }

        Ok(filtered_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_validate_file() {
        let csv_content = "name,age,city\nJohn,30,NYC\nJane,25,LA\n";
        let test_file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(test_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_transform_column() {
        let csv_content = "name,age\nJohn,30\nJane,25\n";
        let test_file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        let results = processor.transform_column(
            test_file.path().to_str().unwrap(),
            0,
            |name| name.to_uppercase()
        ).unwrap();
        
        assert_eq!(results, vec!["JOHN", "JANE"]);
    }

    #[test]
    fn test_filter_rows() {
        let csv_content = "name,age\nJohn,30\nJane,25\nBob,35\n";
        let test_file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        let results = processor.filter_rows(
            test_file.path().to_str().unwrap(),
            |columns| columns[1].parse::<i32>().unwrap_or(0) > 30
        ).unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results[0].contains("name,age"));
        assert!(results[1].contains("Bob,35"));
    }
}