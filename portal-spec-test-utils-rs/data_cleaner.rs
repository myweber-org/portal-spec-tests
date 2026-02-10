use std::collections::HashSet;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn process(&mut self, item: T) -> Option<T> {
        if self.seen.insert(item.clone()) {
            Some(item)
        } else {
            None
        }
    }

    pub fn process_batch(&mut self, items: Vec<T>) -> Vec<T> {
        items
            .into_iter()
            .filter_map(|item| self.process(item))
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }

    pub fn count_unique(&self) -> usize {
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 4, 4, 4, 5];

        let result = cleaner.process_batch(data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.count_unique(), 5);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(cleaner.count_unique(), 3);

        cleaner.reset();
        assert_eq!(cleaner.count_unique(), 0);

        let result = cleaner.process_batch(vec!["a", "b", "c"]);
        assert_eq!(result, vec!["a", "b", "c"]);
    }
}
use std::collections::HashSet;
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_csv(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 {
                writeln!(output_file, "{}", self.clean_header(&line))?;
                continue;
            }

            if let Some(cleaned_line) = self.clean_data_line(&line) {
                writeln!(output_file, "{}", cleaned_line)?;
            }
        }

        Ok(())
    }

    fn clean_header(&self, header: &str) -> String {
        header
            .split(',')
            .map(|col| col.trim().to_lowercase().replace(' ', "_"))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn clean_data_line(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 2 {
            return None;
        }

        let cleaned_parts: Vec<String> = parts
            .iter()
            .map(|&part| {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    "NULL".to_string()
                } else if trimmed.parse::<f64>().is_ok() {
                    trimmed.to_string()
                } else {
                    format!("\"{}\"", trimmed.replace('"', "'"))
                }
            })
            .collect();

        Some(cleaned_parts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "Name, Age, Location\nJohn Doe, 25, New York\nJane, , London\n\"Bob, Jr\", 30, \"Paris, France\"";
        fs::write(test_input, content).unwrap();

        let cleaner = DataCleaner::new(test_input, test_output);
        let result = cleaner.clean_csv();
        
        assert!(result.is_ok());
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("name,age,location"));
        assert!(output_content.contains("NULL"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}