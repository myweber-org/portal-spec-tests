use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub fn compute_file_hash(file_path: &Path) -> Result<String> {
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

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_hash(file_path: &Path, expected_hash: &str) -> Result<bool> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_computation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing.").unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing.").unwrap();

        let hash = compute_file_hash(temp_file.path()).unwrap();
        assert!(verify_file_hash(temp_file.path(), &hash).unwrap());
    }
}