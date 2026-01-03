use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        Ok(Record {
            id: parts[0].trim().parse()?,
            name: parts[1].trim().to_string(),
            value: parts[2].trim().parse()?,
            active: parts[3].trim().parse()?,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }
}

fn filter_records<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    min_value: f64,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    let mut count = 0;
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) if record.value >= min_value => {
                writeln!(output_file, "{}", record.to_csv_line())?;
                count += 1;
            }
            Ok(_) => {}
            Err(e) => eprintln!("Warning: Line {}: {}", line_num + 1, e),
        }
    }

    Ok(count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data.csv";
    let output_file = "filtered.csv";
    let threshold = 100.0;

    match filter_records(input_file, output_file, threshold) {
        Ok(count) => println!("Processed {} matching records", count),
        Err(e) => eprintln!("Error processing file: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let record = Record::from_csv_line("1,Test Item,150.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.value, 150.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_filter_function() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "1,Alpha,99.9,true").unwrap();
        writeln!(input_file, "2,Bravo,100.0,false").unwrap();
        writeln!(input_file, "3,Charlie,100.1,true").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let count = filter_records(
            input_file.path(),
            output_file.path(),
            100.0,
        ).unwrap();

        assert_eq!(count, 2);
    }
}