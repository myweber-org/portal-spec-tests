
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    println!("Enter data to clean (press Ctrl+D when finished):");
    
    let stdin = io::stdin();
    let mut input = String::new();
    
    for line in stdin.lock().lines() {
        match line {
            Ok(text) => input.push_str(&text),
            Err(_) => break,
        }
        input.push('\n');
    }
    
    let cleaned = clean_data(&input);
    
    println!("\nCleaned data:");
    println!("{}", cleaned);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")
        .expect("Failed to create output file");
    
    output_file.write_all(cleaned.as_bytes())
        .expect("Failed to write to output file");
    
    println!("Results saved to cleaned_output.txt");
}
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Vec<T>>,
}

impl<T> DataCleaner<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<Vec<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self, null_value: &T) -> &mut Self {
        self.data.retain(|row| !row.contains(null_value));
        self
    }

    pub fn deduplicate_rows(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|row| seen.insert(row.clone()));
        self
    }

    pub fn get_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    pub fn into_data(self) -> Vec<Vec<T>> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let data = vec![
            vec![Some(1), Some(2)],
            vec![None, Some(3)],
            vec![Some(4), Some(5)],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows(&None);
        let result = cleaner.get_data();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![Some(1), Some(2)]);
        assert_eq!(result[1], vec![Some(4), Some(5)]);
    }

    #[test]
    fn test_deduplicate_rows() {
        let data = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![1, 2, 3],
            vec![7, 8, 9],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate_rows();
        let result = cleaner.get_data();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec![1, 2, 3]);
        assert_eq!(result[1], vec![4, 5, 6]);
        assert_eq!(result[2], vec![7, 8, 9]);
    }
}