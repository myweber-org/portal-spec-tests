
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};
use flate2::write::GzEncoder;
use flate2::Compression;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_LOG_FILES: usize = 5;

pub struct LogProcessor {
    log_dir: PathBuf,
    current_file: String,
}

impl LogProcessor {
    pub fn new(log_dir: &str) -> io::Result<Self> {
        let path = Path::new(log_dir);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        
        let current_file = Self::generate_filename();
        Ok(LogProcessor {
            log_dir: path.to_path_buf(),
            current_file,
        })
    }
    
    fn generate_filename() -> String {
        let now: DateTime<Local> = Local::now();
        format!("app_{}.log", now.format("%Y%m%d_%H%M%S"))
    }
    
    pub fn write_log(&mut self, message: &str) -> io::Result<()> {
        let log_path = self.log_dir.join(&self.current_file);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        let now: DateTime<Local> = Local::now();
        let log_entry = format!("[{}] {}\n", now.format("%Y-%m-%d %H:%M:%S"), message);
        file.write_all(log_entry.as_bytes())?;
        
        self.check_rotation(&log_path)?;
        Ok(())
    }
    
    fn check_rotation(&mut self, current_path: &Path) -> io::Result<()> {
        let metadata = fs::metadata(current_path)?;
        if metadata.len() >= MAX_LOG_SIZE {
            self.rotate_logs()?;
            self.current_file = Self::generate_filename();
        }
        Ok(())
    }
    
    fn rotate_logs(&self) -> io::Result<()> {
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file() && 
                path.extension().map_or(false, |ext| ext == "log" || ext == "gz")
            })
            .collect();
        
        log_files.sort_by_key(|path| {
            fs::metadata(path).ok().and_then(|m| m.modified().ok())
        });
        
        while log_files.len() >= MAX_LOG_FILES {
            if let Some(oldest) = log_files.first() {
                fs::remove_file(oldest)?;
                log_files.remove(0);
            }
        }
        
        self.compress_old_logs()?;
        Ok(())
    }
    
    fn compress_old_logs(&self) -> io::Result<()> {
        for entry in fs::read_dir(&self.log_dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                let compressed_path = path.with_extension("log.gz");
                if !compressed_path.exists() {
                    self.compress_file(&path, &compressed_path)?;
                    fs::remove_file(&path)?;
                }
            }
        }
        Ok(())
    }
    
    fn compress_file(&self, source: &Path, dest: &Path) -> io::Result<()> {
        let source_file = File::open(source)?;
        let reader = BufReader::new(source_file);
        let dest_file = File::create(dest)?;
        let mut encoder = GzEncoder::new(dest_file, Compression::default());
        
        for line in reader.lines() {
            let line = line?;
            writeln!(encoder, "{}", line)?;
        }
        
        encoder.finish()?;
        Ok(())
    }
    
    pub fn read_recent_logs(&self, count: usize) -> io::Result<Vec<String>> {
        let mut logs = Vec::new();
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();
        
        log_files.sort_by_key(|path| {
            fs::metadata(path).ok().and_then(|m| m.modified().ok())
        });
        
        for path in log_files.iter().rev().take(count) {
            if path.extension().map_or(false, |ext| ext == "gz") {
                if let Ok(decompressed) = self.decompress_file(path) {
                    logs.extend(decompressed);
                }
            } else {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    logs.push(line?);
                }
            }
        }
        
        Ok(logs.into_iter().rev().take(count).collect())
    }
    
    fn decompress_file(&self, path: &Path) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        let mut lines = Vec::new();
        
        for line in reader.lines() {
            lines.push(line?);
        }
        
        Ok(lines)
    }
}