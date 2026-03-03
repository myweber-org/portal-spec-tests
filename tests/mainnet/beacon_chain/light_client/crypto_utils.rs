use sha2::{Digest, Sha256};
use std::error::Error;

pub fn compute_sha256(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    Ok(result.to_vec())
}

pub fn compute_sha256_hex(input: &[u8]) -> Result<String, Box<dyn Error>> {
    let hash_bytes = compute_sha256(input)?;
    let hex_string = hash_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();
    Ok(hex_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_basic() {
        let data = b"hello world";
        let hash = compute_sha256(data).unwrap();
        assert_eq!(hash.len(), 32);
        
        let hex_hash = compute_sha256_hex(data).unwrap();
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hex_hash, expected);
    }

    #[test]
    fn test_sha256_empty() {
        let data = b"";
        let hash = compute_sha256(data).unwrap();
        assert_eq!(hash.len(), 32);
        
        let hex_hash = compute_sha256_hex(data).unwrap();
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hex_hash, expected);
    }
}