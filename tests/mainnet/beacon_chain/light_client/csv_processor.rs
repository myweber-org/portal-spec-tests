
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Record>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Record, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(format!("Invalid number of fields at line {}", line_num).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    pub fn validate_records(&self, records: &[Record]) -> Vec<String> {
        let mut errors = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            if record.name.is_empty() {
                errors.push(format!("Record {}: Name cannot be empty", idx + 1));
            }
            
            if record.value < 0.0 {
                errors.push(format!("Record {}: Value cannot be negative", idx + 1));
            }
        }

        errors
    }

    pub fn calculate_statistics(&self, records: &[Record]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[0].value, 10.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            Record { id: 1, name: "".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Valid".to_string(), value: -5.0, active: false },
        ];

        let processor = CsvProcessor::new(',', false);
        let errors = processor.validate_records(&records);

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Name cannot be empty"));
        assert!(errors[1].contains("Value cannot be negative"));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let processor = CsvProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&records);

        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn filter_csv_rows(
    input_path: &str,
    output_path: &str,
    predicate: impl Fn(&[String]) -> bool,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let mut processed_count = 0;
    for result in csv_reader.records() {
        let record = result?;
        let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        if predicate(&fields) {
            csv_writer.write_record(&fields)?;
            processed_count += 1;
        }
    }

    csv_writer.flush()?;
    Ok(processed_count)
}

pub fn transform_csv_column(
    input_path: &str,
    output_path: &str,
    column_index: usize,
    transformer: impl Fn(&str) -> String,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let mut record = result?;
        if column_index < record.len() {
            let transformed_value = transformer(&record[column_index]);
            record[column_index] = transformed_value.into();
        }
        csv_writer.write_record(&record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_csv_rows() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,25,London").unwrap();
        writeln!(input_file, "Charlie,35,Tokyo").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let result = filter_csv_rows(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            |fields| fields[1].parse::<i32>().unwrap_or(0) >= 30,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_transform_csv_column() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age").unwrap();
        writeln!(input_file, "alice,30").unwrap();
        writeln!(input_file, "bob,25").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let result = transform_csv_column(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            0,
            |s| s.to_uppercase(),
        );

        assert!(result.is_ok());
    }
}