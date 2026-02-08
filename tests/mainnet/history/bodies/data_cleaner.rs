
use regex::Regex;
use std::collections::HashSet;

pub fn clean_and_normalize(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let normalized_spaces = re_multispace.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-\.]").unwrap();
    let cleaned = re_special.replace_all(&normalized_spaces, "");
    
    cleaned.to_lowercase()
}

pub fn remove_duplicates(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_normalize() {
        assert_eq!(
            clean_and_normalize("  Hello   WORLD!!  "),
            "hello world"
        );
    }

    #[test]
    fn test_remove_duplicates() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = remove_duplicates(input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut wtr = WriterBuilder::new().from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

pub fn clean_csv_from_stdin(output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(io::stdin());
    let mut wtr = WriterBuilder::new().from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if record.iter().all(|field| !field.trim().is_empty()) {
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}use std::collections::HashMap;

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn remove_outliers(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_quartile(&sorted_data, 0.25);
        let q3 = Self::calculate_quartile(&sorted_data, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        data.iter()
            .filter(|&&value| value >= lower_bound && value <= upper_bound)
            .cloned()
            .collect()
    }

    fn calculate_quartile(sorted_data: &[f64], percentile: f64) -> f64 {
        let index = percentile * (sorted_data.len() - 1) as f64;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;

        if lower_index == upper_index {
            sorted_data[lower_index]
        } else {
            let weight = index - lower_index as f64;
            sorted_data[lower_index] * (1.0 - weight) + sorted_data[upper_index] * weight
        }
    }

    pub fn analyze_dataset(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !data.is_empty() {
            let sum: f64 = data.iter().sum();
            let mean = sum / data.len() as f64;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
            let std_dev = variance.sqrt();

            stats.insert("mean".to_string(), mean);
            stats.insert("std_dev".to_string(), std_dev);
            stats.insert("min".to_string(), *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
            stats.insert("max".to_string(), *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaned = cleaner.remove_outliers(&data);
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = cleaner.analyze_dataset(&data);
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("min").unwrap(), &1.0);
        assert_eq!(stats.get("max").unwrap(), &5.0);
    }
}