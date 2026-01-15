use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct ColumnStats {
    name: String,
    count: u32,
    sum: f64,
    min: f64,
    max: f64,
}

impl ColumnStats {
    fn new(name: String) -> Self {
        ColumnStats {
            name,
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }
}

fn analyze_csv(file_path: &str) -> Result<Vec<ColumnStats>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty file or missing header".into()),
    };

    let column_names: Vec<String> = header
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut stats_map: HashMap<String, ColumnStats> = HashMap::new();
    for name in &column_names {
        stats_map.insert(name.clone(), ColumnStats::new(name.clone()));
    }

    for line_result in lines {
        let line = line_result?;
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if values.len() != column_names.len() {
            continue;
        }

        for (i, value_str) in values.iter().enumerate() {
            if let Ok(value) = value_str.parse::<f64>() {
                if let Some(stats) = stats_map.get_mut(&column_names[i]) {
                    stats.update(value);
                }
            }
        }
    }

    let mut results: Vec<ColumnStats> = stats_map.into_values().collect();
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    let stats = analyze_csv(file_path)?;

    for stat in stats {
        println!("Column: {}", stat.name);
        println!("  Count: {}", stat.count);
        println!("  Sum: {:.2}", stat.sum);
        println!("  Average: {:.2}", stat.average());
        println!("  Min: {:.2}", stat.min);
        println!("  Max: {:.2}", stat.max);
        println!();
    }

    Ok(())
}