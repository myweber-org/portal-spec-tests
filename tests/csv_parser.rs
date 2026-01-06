use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_csv(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();
        println!("{:?}", fields);
    }

    Ok(())
}