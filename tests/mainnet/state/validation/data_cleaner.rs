
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_outliers_iqr(&self, factor: f64) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - factor * iqr;
        let upper_bound = q3 + factor * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaner = DataCleaner::new(data);
        let cleaned = cleaner.remove_outliers_iqr(1.5);

        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.calculate_statistics();

        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("count"), Some(&5.0));
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub items: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    pub fn remove_duplicates(&mut self) {
        let mut seen = HashSet::new();
        self.items.retain(|item| seen.insert(item.clone()));
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.items {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn clean(&mut self) {
        self.normalize_strings();
        self.remove_duplicates();
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("  Apple  ");
        cleaner.add_item("apple");
        cleaner.add_item("Banana");
        cleaner.add_item("banana ");
        cleaner.add_item("Orange");

        cleaner.clean();
        let items = cleaner.get_items();

        assert_eq!(items.len(), 3);
        assert!(items.contains(&"apple".to_string()));
        assert!(items.contains(&"banana".to_string()));
        assert!(items.contains(&"orange".to_string()));
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr_thresholds(&mut self) -> (f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower".to_string(), lower_bound);
        self.thresholds.insert("upper".to_string(), upper_bound);

        (lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = self.thresholds.get("lower").unwrap_or(&f64::MIN);
        let upper = self.thresholds.get("upper").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&x| x >= *lower && x <= *upper)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), self.data.len() as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        
        cleaner.calculate_iqr_thresholds();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        
        let stats = cleaner.get_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    io::stdout().write_all(cleaned.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}