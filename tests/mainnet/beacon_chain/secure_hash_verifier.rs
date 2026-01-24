use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};

pub fn calculate_sha256(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_integrity(file_path: &str, expected_hash: &str) -> Result<bool> {
    let actual_hash = calculate_sha256(file_path)?;
    Ok(actual_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_sha256_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = calculate_sha256(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test").unwrap();
        
        let hash = calculate_sha256(temp_file.path().to_str().unwrap()).unwrap();
        let verified = verify_file_integrity(temp_file.path().to_str().unwrap(), &hash).unwrap();
        assert!(verified);
    }
}