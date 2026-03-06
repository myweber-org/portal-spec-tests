
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileVerifier;

impl FileVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

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

    pub fn verify_file<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hash verification").unwrap();
        
        let hash = FileVerifier::calculate_sha256(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Consistent test content").unwrap();
        
        let hash = FileVerifier::calculate_sha256(temp_file.path()).unwrap();
        let is_valid = FileVerifier::verify_file(temp_file.path(), &hash).unwrap();
        
        assert!(is_valid);
    }
}
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
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

    pub fn verify_file_hash(file_path: &Path, expected_hash: &str) -> Result<bool> {
        let calculated_hash = Self::calculate_file_hash(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files(file1: &Path, file2: &Path) -> Result<bool> {
        let hash1 = Self::calculate_file_hash(file1)?;
        let hash2 = Self::calculate_file_hash(file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        assert!(HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Data 1")?;
        file2.write_all(b"Data 2")?;
        
        assert!(!HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Test content")?;
        
        let hash = HashVerifier::calculate_file_hash(file.path())?;
        assert!(HashVerifier::verify_file_hash(file.path(), &hash)?);
        Ok(())
    }
}