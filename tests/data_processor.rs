use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, Box<dyn Error>> {
        if value < 0.0 {
            return Err("Value cannot be negative".into());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".into());
        }
        Ok(Self { id, value, category })
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].trim().to_string();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_record_negative_value() {
        let record = DataRecord::new(1, -10.0, "B".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.0,Alpha").unwrap();
        writeln!(temp_file, "2,200.0,Beta").unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(temp_file, "3,300.0,Gamma").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].category, "Beta");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, "X".to_string()).unwrap(),
            DataRecord::new(2, 20.0, "Y".to_string()).unwrap(),
            DataRecord::new(3, 30.0, "Z".to_string()).unwrap(),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}