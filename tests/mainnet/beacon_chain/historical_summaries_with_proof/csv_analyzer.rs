use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P) -> Result<Vec<ColumnStats>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let mut column_data: Vec<Vec<f64>> = vec![Vec::new(); headers.len()];
    
    for result in rdr.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if let Ok(value) = field.parse::<f64>() {
                column_data[i].push(value);
            }
        }
    }
    
    let mut stats = Vec::new();
    for (i, data) in column_data.iter().enumerate() {
        if !data.is_empty() {
            let col_stats = calculate_stats(&headers[i], data);
            stats.push(col_stats);
        }
    }
    
    Ok(stats)
}

fn calculate_stats(name: &str, data: &[f64]) -> ColumnStats {
    let count = data.len();
    let sum: f64 = data.iter().sum();
    let mean = sum / count as f64;
    
    let min = *data.iter().fold(&f64::INFINITY, |a, &b| a.min(&b));
    let max = *data.iter().fold(&f64::NEG_INFINITY, |a, &b| a.max(&b));
    
    let variance: f64 = data.iter()
        .map(|value| {
            let diff = mean - value;
            diff * diff
        })
        .sum::<f64>() / count as f64;
    
    let std_dev = variance.sqrt();
    
    ColumnStats {
        name: name.to_string(),
        count,
        mean,
        min,
        max,
        std_dev,
    }
}

pub fn print_summary(stats: &[ColumnStats]) {
    println!("CSV Analysis Summary:");
    println!("{:<20} {:<10} {:<10} {:<10} {:<10} {:<10}", 
             "Column", "Count", "Mean", "Min", "Max", "Std Dev");
    println!("{}", "-".repeat(80));
    
    for stat in stats {
        println!("{:<20} {:<10} {:<10.4} {:<10.4} {:<10.4} {:<10.4}",
                 stat.name, stat.count, stat.mean, stat.min, stat.max, stat.std_dev);
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
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.0,2.0,3.0").unwrap();
        writeln!(temp_file, "4.0,5.0,6.0").unwrap();
        writeln!(temp_file, "7.0,8.0,9.0").unwrap();
        
        let stats = analyze_csv(temp_file.path()).unwrap();
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].name, "col1");
        assert_eq!(stats[0].count, 3);
        assert!((stats[0].mean - 4.0).abs() < 0.001);
    }
}