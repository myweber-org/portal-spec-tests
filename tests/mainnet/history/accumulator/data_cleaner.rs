use std::collections::HashSet;

pub fn clean_data<T: Ord + Clone + std::hash::Hash>(data: &[T]) -> Vec<T> {
    let mut unique_set: HashSet<T> = HashSet::new();
    for item in data {
        unique_set.insert(item.clone());
    }
    
    let mut result: Vec<T> = unique_set.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let expected = vec![1, 2, 5, 8, 9];
        assert_eq!(clean_data(&input), expected);
    }

    #[test]
    fn test_clean_data_strings() {
        let input = vec!["apple", "banana", "apple", "cherry"];
        let expected = vec!["apple", "banana", "cherry"];
        assert_eq!(clean_data(&input), expected);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let cleaned_line = clean_csv_line(&line);
        writeln!(output_file, "{}", cleaned_line)?;

        if line_num % 1000 == 0 {
            println!("Processed {} lines", line_num);
        }
    }

    println!("CSV cleaning completed successfully");
    Ok(())
}

fn clean_csv_line(line: &str) -> String {
    let cleaned_columns: Vec<String> = line
        .split(',')
        .map(|column| column.trim().to_string())
        .collect();
    cleaned_columns.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_csv_line() {
        let dirty_line = "  apple, banana  ,  cherry , durian  ";
        let cleaned = clean_csv_line(dirty_line);
        assert_eq!(cleaned, "apple,banana,cherry,durian");
    }

    #[test]
    fn test_clean_csv_line_empty() {
        assert_eq!(clean_csv_line(""), "");
        assert_eq!(clean_csv_line("   "), "");
    }
}