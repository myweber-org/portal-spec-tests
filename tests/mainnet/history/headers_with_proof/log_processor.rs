use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_LOG_FILES: usize = 5;

pub struct LogProcessor {
    log_dir: PathBuf,
    current_file: String,
}

impl LogProcessor {
    pub fn new(log_dir: impl AsRef<Path>) -> io::Result<Self> {
        let log_dir = log_dir.as_ref().to_path_buf();
        fs::create_dir_all(&log_dir)?;
        
        let current_file = format!("application_{}.log", Local::now().format("%Y%m%d_%H%M%S"));
        
        Ok(Self {
            log_dir,
            current_file,
        })
    }
    
    pub fn write_log(&mut self, message: &str) -> io::Result<()> {
        let log_path = self.log_dir.join(&self.current_file);
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
            
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(file, "[{}] {}", timestamp, message)?;
        
        self.check_rotation(&log_path)?;
        
        Ok(())
    }
    
    fn check_rotation(&mut self, current_path: &Path) -> io::Result<()> {
        let metadata = fs::metadata(current_path)?;
        
        if metadata.len() >= MAX_LOG_SIZE {
            self.rotate_log(current_path)?;
        }
        
        self.cleanup_old_logs()?;
        
        Ok(())
    }
    
    fn rotate_log(&mut self, current_path: &Path) -> io::Result<()> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("application_{}.log.gz", timestamp);
        let archive_path = self.log_dir.join(archive_name);
        
        let log_file = File::open(current_path)?;
        let archive_file = File::create(&archive_path)?;
        let mut encoder = GzEncoder::new(archive_file, Compression::default());
        
        let mut reader = BufReader::new(log_file);
        let mut buffer = Vec::new();
        
        reader.read_to_end(&mut buffer)?;
        encoder.write_all(&buffer)?;
        encoder.finish()?;
        
        fs::remove_file(current_path)?;
        
        self.current_file = format!("application_{}.log", Local::now().format("%Y%m%d_%H%M%S"));
        
        Ok(())
    }
    
    fn cleanup_old_logs(&self) -> io::Result<()> {
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().map_or(false, |ext| ext == "gz") &&
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| name.starts_with("application_"))
            })
            .collect();
            
        log_files.sort_by_key(|path| {
            fs::metadata(path)
                .and_then(|md| md.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        
        while log_files.len() > MAX_LOG_FILES {
            if let Some(oldest) = log_files.first() {
                fs::remove_file(oldest)?;
                log_files.remove(0);
            }
        }
        
        Ok(())
    }
    
    pub fn search_logs(&self, pattern: &str) -> io::Result<Vec<String>> {
        let mut results = Vec::new();
        
        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "log") {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                
                for line in reader.lines() {
                    let line = line?;
                    if line.contains(pattern) {
                        results.push(format!("{}: {}", path.display(), line));
                    }
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_log_rotation() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let mut processor = LogProcessor::new(temp_dir.path())?;
        
        for i in 0..1000 {
            processor.write_log(&format!("Test message {}", i))?;
        }
        
        let gz_files = fs::read_dir(temp_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .map_or(false, |ext| ext == "gz")
            })
            .count();
            
        assert!(gz_files > 0, "Should have created compressed log files");
        
        Ok(())
    }
    
    #[test]
    fn test_log_search() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let mut processor = LogProcessor::new(temp_dir.path())?;
        
        processor.write_log("Error: Database connection failed")?;
        processor.write_log("Info: User logged in")?;
        processor.write_log("Error: File not found")?;
        
        let results = processor.search_logs("Error")?;
        assert_eq!(results.len(), 2);
        
        Ok(())
    }
}