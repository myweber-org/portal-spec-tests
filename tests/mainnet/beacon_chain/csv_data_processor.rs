
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|record| record.value).reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|record| record.value).reduce(f64::min)
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.add_record(record);
        }

        Ok(())
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn summary(&self) -> String {
        format!(
            "Records: {}, Active: {}, Avg Value: {:.2}, Max: {:.2}, Min: {:.2}",
            self.records.len(),
            self.filter_active().len(),
            self.average_value().unwrap_or(0.0),
            self.max_value().unwrap_or(0.0),
            self.min_value().unwrap_or(0.0)
        )
    }
}

pub fn process_data_file(input_path: &str, output_path: &str) -> Result<String, Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv(input_path)?;

    let active_records = processor.filter_active();
    let tech_records = processor.filter_by_category("Technology");

    let summary = processor.summary();
    processor.save_to_csv(output_path)?;

    Ok(format!(
        "Processed {} records. Active: {}, Technology: {}. Summary: {}",
        processor.records.len(),
        active_records.len(),
        tech_records.len(),
        summary
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        processor.add_record(DataRecord::new(1, "Technology".to_string(), 100.0, true));
        processor.add_record(DataRecord::new(2, "Finance".to_string(), 200.0, true));
        processor.add_record(DataRecord::new(3, "Technology".to_string(), 150.0, false));

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_by_category("Technology").len(), 2);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.average_value(), Some(150.0));
        assert_eq!(processor.max_value(), Some(200.0));
        assert_eq!(processor.min_value(), Some(100.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
        assert_eq!(processor.average_value(), None);
        assert_eq!(processor.max_value(), None);
        assert_eq!(processor.min_value(), None);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let record = Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        };
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,299.99,true").unwrap();
        writeln!(temp_file, "2,books,19.99,false").unwrap();

        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "electronics");
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                category: "electronics".to_string(),
                value: 299.99,
                active: true,
            },
            Record {
                id: 2,
                category: "books".to_string(),
                value: 19.99,
                active: false,
            },
        ];

        let filtered = filter_by_category(&records, "electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                category: "test".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                category: "test".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let avg = calculate_average(&records).unwrap();
        assert_eq!(avg, 15.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            _ => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err("Column not found".into()),
        };

        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found".into())
        }
    }

    pub fn group_by_column(&self, group_column: &str, aggregate_column: &str) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return Err("Group column not found".into()),
        };

        let agg_idx = match self.headers.iter().position(|h| h == aggregate_column) {
            Some(idx) => idx,
            None => return Err("Aggregate column not found".into()),
        };

        let mut groups: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            if let (Some(group_val), Some(agg_val_str)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(agg_val) = agg_val_str.parse::<f64>() {
                    let entry = groups.entry(group_val.clone()).or_insert((0.0, 0));
                    entry.0 += agg_val;
                    entry.1 += 1;
                }
            }
        }

        let result: HashMap<String, f64> = groups
            .into_iter()
            .map(|(key, (sum, count))| (key, sum / count as f64))
            .collect();

        Ok(result)
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,department,salary").unwrap();
        writeln!(file, "Alice,Engineering,75000").unwrap();
        writeln!(file, "Bob,Marketing,65000").unwrap();
        writeln!(file, "Charlie,Engineering,80000").unwrap();
        writeln!(file, "Diana,Marketing,70000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 4);
        assert_eq!(processor.get_headers(), &vec!["name".to_string(), "department".to_string(), "salary".to_string()]);
    }

    #[test]
    fn test_filter_by_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 72500.0).abs() < 0.001);
    }

    #[test]
    fn test_group_by_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(file.path().to_str().unwrap()).unwrap();
        let dept_avg_salary = processor.group_by_column("department", "salary").unwrap();
        assert!((dept_avg_salary["Engineering"] - 77500.0).abs() < 0.001);
        assert!((dept_avg_salary["Marketing"] - 67500.0).abs() < 0.001);
    }
}