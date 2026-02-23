
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use sha2::{Sha256, Digest};
use indicatif::{ProgressBar, ProgressStyle};

pub fn calculate_file_hash(file_path: &Path) -> Result<String, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let metadata = file.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let file_size = metadata.len();
    let pb = ProgressBar::new(file_size);
    
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    let mut hasher = Sha256::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
        pb.inc(bytes_read as u64);
    }
    
    pb.finish_with_message("Hash calculation complete");
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool, String> {
    let calculated_hash = calculate_file_hash(file_path)?;
    
    if calculated_hash == expected_hash.to_lowercase() {
        println!("✓ File integrity verified successfully");
        Ok(true)
    } else {
        println!("✗ File integrity check failed");
        println!("  Expected: {}", expected_hash);
        println!("  Got:      {}", calculated_hash);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hash verification").unwrap();
        
        let hash_result = calculate_file_hash(temp_file.path());
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();
        
        let hash = calculate_file_hash(temp_file.path()).unwrap();
        let verification = verify_file_integrity(temp_file.path(), &hash);
        
        assert!(verification.is_ok());
        assert_eq!(verification.unwrap(), true);
    }
}