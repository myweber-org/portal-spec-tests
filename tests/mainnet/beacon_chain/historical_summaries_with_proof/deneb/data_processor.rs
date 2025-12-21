use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_threshold: f64) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                filtered_data.push(line.split(',').map(|s| s.to_string()).collect());
                continue;
            }

            let columns: Vec<&str> = line.split(',').collect();
            if columns.len() >= 3 {
                if let Ok(value) = columns[2].parse::<f64>() {
                    if value > filter_threshold {
                        filtered_data.push(columns.iter().map(|&s| s.to_string()).collect());
                    }
                }
            }
        }

        Ok(filtered_data)
    }

    pub fn calculate_statistics(&self, column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let columns: Vec<&str> = line.split(',').collect();
            if column_index < columns.len() {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return Ok((0.0, 0.0));
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item_a,15.5").unwrap();
        writeln!(temp_file, "2,item_b,8.2").unwrap();
        writeln!(temp_file, "3,item_c,22.7").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(10.0).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[1][2], "15.5");
        assert_eq!(result[2][2], "22.7");
    }

    #[test]
    fn test_calculate_statistics() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,score").unwrap();
        writeln!(temp_file, "1,85.0").unwrap();
        writeln!(temp_file, "2,92.0").unwrap();
        writeln!(temp_file, "3,78.0").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let (mean, std_dev) = processor.calculate_statistics(1).unwrap();

        assert!((mean - 85.0).abs() < 0.1);
        assert!(std_dev > 0.0);
    }
}