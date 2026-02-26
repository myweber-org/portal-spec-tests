
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str, has_headers: bool) -> Result<(), Box<dyn Error>> {
    let mut unique_lines = HashSet::new();
    let mut headers = Vec::new();
    
    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if has_headers {
            if let Some(header) = lines.next() {
                let header_line = header?;
                if headers.is_empty() {
                    headers.push(header_line.clone());
                }
                unique_lines.insert(header_line);
            }
        }
        
        for line_result in lines {
            let line = line_result?;
            unique_lines.insert(line);
        }
    }
    
    let mut sorted_lines: Vec<String> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    if has_headers && !headers.is_empty() {
        if let Some(pos) = sorted_lines.iter().position(|line| line == &headers[0]) {
            let header = sorted_lines.remove(pos);
            sorted_lines.insert(0, header);
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(output_file);
    
    for line in sorted_lines {
        writeln!(writer, "{}", line)?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_merge_csv_files() {
        let test_dir = "test_data";
        fs::create_dir_all(test_dir).unwrap();
        
        let file1_content = "id,name,value\n1,alpha,100\n2,beta,200";
        let file2_content = "id,name,value\n3,gamma,300\n1,alpha,100";
        
        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);
        
        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();
        
        let input_paths = [file1_path.as_str(), file2_path.as_str()];
        let result = merge_csv_files(&input_paths, &output_path, true);
        
        assert!(result.is_ok());
        
        let merged_content = fs::read_to_string(&output_path).unwrap();
        let expected_lines = vec![
            "id,name,value",
            "1,alpha,100",
            "2,beta,200",
            "3,gamma,300"
        ];
        
        for expected_line in expected_lines {
            assert!(merged_content.contains(expected_line));
        }
        
        fs::remove_dir_all(test_dir).unwrap();
    }
}