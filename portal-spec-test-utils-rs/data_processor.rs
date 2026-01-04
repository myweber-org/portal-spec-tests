use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use log::{error, info, warn};

pub struct DataProcessor {
    input_path: String,
    output_path: String,
}

impl DataProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Starting data processing from {} to {}", self.input_path, self.output_path);
        
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut writer = io::BufWriter::new(output_file);
        
        let mut processed_count = 0;
        
        for (line_num, line_result) in reader.lines().enumerate() {
            match line_result {
                Ok(line) => {
                    let processed_line = self.transform_line(&line);
                    writeln!(writer, "{}", processed_line)?;
                    processed_count += 1;
                    
                    if line_num % 1000 == 0 {
                        info!("Processed {} lines", line_num);
                    }
                }
                Err(e) => {
                    warn!("Error reading line {}: {}", line_num + 1, e);
                    continue;
                }
            }
        }
        
        writer.flush()?;
        info!("Completed processing. Total lines processed: {}", processed_count);
        
        Ok(processed_count)
    }
    
    fn transform_line(&self, line: &str) -> String {
        line.trim()
            .to_uppercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }
    
    pub fn validate_paths(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.input_path).exists() {
            return Err(format!("Input file does not exist: {}", self.input_path).into());
        }
        
        let output_dir = Path::new(&self.output_path).parent();
        if let Some(dir) = output_dir {
            if !dir.exists() {
                return Err(format!("Output directory does not exist: {}", dir.display()).into());
            }
        }
        
        Ok(())
    }
}

pub fn initialize_logger() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .try_init()?;
    Ok(())
}