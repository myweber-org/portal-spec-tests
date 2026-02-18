use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
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

pub fn verify_file_integrity<P: AsRef<Path>>(path: P, expected_hash: &str) -> io::Result<bool> {
    let computed_hash = compute_file_hash(path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for hashing";
        temp_file.write_all(test_data).unwrap();

        let hash1 = compute_file_hash(temp_file.path()).unwrap();
        let hash2 = compute_file_hash(temp_file.path()).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Verification test";
        temp_file.write_all(test_data).unwrap();

        let computed_hash = compute_file_hash(temp_file.path()).unwrap();
        let verification_result = verify_file_integrity(temp_file.path(), &computed_hash).unwrap();

        assert!(verification_result);
    }
}