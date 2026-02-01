use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub fn compute_file_hash(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_consistency() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Test data for hash verification";
        temp_file.write_all(test_data)?;
        
        let hash1 = compute_file_hash(temp_file.path())?;
        let hash2 = compute_file_hash(temp_file.path())?;
        
        assert_eq!(hash1, hash2);
        Ok(())
    }

    #[test]
    fn test_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Verification test content";
        temp_file.write_all(test_data)?;
        
        let computed_hash = compute_file_hash(temp_file.path())?;
        assert!(verify_file_integrity(temp_file.path(), &computed_hash)?);
        assert!(!verify_file_integrity(temp_file.path(), "invalidhash123")?);
        Ok(())
    }
}