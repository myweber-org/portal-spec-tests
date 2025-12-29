use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        
        let min = self.records
            .iter()
            .map(|r| r.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
            
        let max = self.records
            .iter()
            .map(|r| r.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        (count, avg, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = "id,name,value,category\n\
                        1,ItemA,10.5,Electronics\n\
                        2,ItemB,15.0,Electronics\n\
                        3,ItemC,8.75,Books";
        
        let temp_file = "test_data.csv";
        std::fs::write(temp_file, test_data).unwrap();
        
        let result = processor.load_from_csv(temp_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 11.416666).abs() < 0.001);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        
        std::fs::remove_file(temp_file).unwrap();
    }
}