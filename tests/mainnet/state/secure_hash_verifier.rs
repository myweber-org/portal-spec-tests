
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};

pub struct FileVerifier {
    buffer_size: usize,
}

impl FileVerifier {
    pub fn new(buffer_size: usize) -> Self {
        FileVerifier { buffer_size }
    }

    pub fn calculate_sha256(&self, file_path: &Path) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let file_size = file.metadata()?.len();
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.buffer_size];
        let mut bytes_processed = 0u64;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
            bytes_processed += bytes_read as u64;
            
            let progress = (bytes_processed as f64 / file_size as f64) * 100.0;
            print!("\rProcessing: {:.1}%", progress);
            io::Write::flush(&mut io::stdout())?;
        }
        
        println!();
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_file(&self, file_path: &Path, expected_hash: &str) -> io::Result<bool> {
        let calculated_hash = self.calculate_sha256(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }
}

pub fn verify_files_concurrently(
    file_hashes: Vec<(&Path, &str)>,
    buffer_size: usize,
) -> Vec<(String, bool)> {
    use std::sync::Arc;
    use std::thread;
    
    let verifier = Arc::new(FileVerifier::new(buffer_size));
    let mut handles = vec![];
    
    for (file_path, expected_hash) in file_hashes {
        let verifier_clone = Arc::clone(&verifier);
        let path = file_path.to_path_buf();
        let hash = expected_hash.to_string();
        
        let handle = thread::spawn(move || {
            match verifier_clone.verify_file(&path, &hash) {
                Ok(is_valid) => (path.display().to_string(), is_valid),
                Err(e) => (path.display().to_string(), false),
            }
        });
        
        handles.push(handle);
    }
    
    let mut results = vec![];
    for handle in handles {
        match handle.join() {
            Ok(result) => results.push(result),
            Err(_) => results.push(("Thread panicked".to_string(), false)),
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let verifier = FileVerifier::new(4096);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hashing").unwrap();
        
        let hash = verifier.calculate_sha256(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_file_verification() {
        let verifier = FileVerifier::new(4096);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();
        
        let hash = verifier.calculate_sha256(temp_file.path()).unwrap();
        let is_valid = verifier.verify_file(temp_file.path(), &hash).unwrap();
        
        assert!(is_valid);
    }
}