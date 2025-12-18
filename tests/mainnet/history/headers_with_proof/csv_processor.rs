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
}