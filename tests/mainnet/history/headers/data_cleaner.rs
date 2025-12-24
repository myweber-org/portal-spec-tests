use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid CSV file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(output_path)?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv_duplicates() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        fs::write(test_input, test_data).unwrap();
        
        clean_csv_duplicates(test_input, test_output).unwrap();
        
        let result = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n";
        
        assert_eq!(result, expected);
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}