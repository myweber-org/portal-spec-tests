
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_types: HashMap<String, String>,
    pub missing_values: usize,
    pub unique_values: HashMap<String, usize>,
}

impl CsvStats {
    pub fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_types: HashMap::new(),
            missing_values: 0,
            unique_values: HashMap::new(),
        }
    }
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut stats = CsvStats::new();
    let mut headers: Vec<String> = Vec::new();
    let mut data_types: HashMap<String, Vec<String>> = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            headers = line.split(',').map(|s| s.trim().to_string()).collect();
            stats.column_count = headers.len();
            for header in &headers {
                data_types.insert(header.clone(), Vec::new());
            }
            continue;
        }

        stats.row_count += 1;
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        for (i, value) in values.iter().enumerate() {
            if i >= headers.len() {
                break;
            }

            let header = &headers[i];
            
            if value.is_empty() {
                stats.missing_values += 1;
                continue;
            }

            if let Some(type_list) = data_types.get_mut(header) {
                type_list.push(value.to_string());
            }

            let value_key = format!("{}_{}", header, value);
            *stats.unique_values.entry(value_key).or_insert(0) += 1;
        }
    }

    for (header, values) in data_types {
        let inferred_type = infer_data_type(&values);
        stats.column_types.insert(header, inferred_type);
    }

    Ok(stats)
}

fn infer_data_type(values: &[String]) -> String {
    if values.is_empty() {
        return "unknown".to_string();
    }

    let mut is_numeric = true;
    let mut is_integer = true;
    let mut has_decimal = false;

    for value in values {
        if value.parse::<i64>().is_err() {
            is_integer = false;
            if value.parse::<f64>().is_err() {
                is_numeric = false;
                break;
            } else {
                has_decimal = true;
            }
        }
    }

    if is_numeric {
        if is_integer && !has_decimal {
            "integer".to_string()
        } else {
            "float".to_string()
        }
    } else {
        "string".to_string()
    }
}

pub fn print_stats(stats: &CsvStats) {
    println!("CSV Analysis Results:");
    println!("Rows: {}", stats.row_count);
    println!("Columns: {}", stats.column_count);
    println!("Missing Values: {}", stats.missing_values);
    
    println!("\nColumn Types:");
    for (column, data_type) in &stats.column_types {
        println!("  {}: {}", column, data_type);
    }
    
    println!("\nUnique Value Counts (sample):");
    let mut sample: Vec<(&String, &usize)> = stats.unique_values.iter().take(5).collect();
    sample.sort_by(|a, b| b.1.cmp(a.1));
    
    for (key, count) in sample {
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() >= 2 {
            println!("  {}: {} occurrences", parts[1], count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,John,25,50000.50").unwrap();
        writeln!(temp_file, "2,Jane,30,").unwrap();
        writeln!(temp_file, "3,Bob,35,75000").unwrap();

        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.missing_values, 1);
        
        assert_eq!(stats.column_types.get("id").unwrap(), "integer");
        assert_eq!(stats.column_types.get("name").unwrap(), "string");
        assert_eq!(stats.column_types.get("age").unwrap(), "integer");
        assert_eq!(stats.column_types.get("salary").unwrap(), "float");
    }
}