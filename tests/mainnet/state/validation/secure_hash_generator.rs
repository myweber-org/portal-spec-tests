
use sha2::{Digest, Sha256};
use std::error::Error;

pub fn generate_secure_hash(input: &str) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    
    Ok(format!("{:x}", result))
}

pub fn validate_hash(input: &str, expected_hash: &str) -> Result<bool, Box<dyn Error>> {
    let generated_hash = generate_secure_hash(input)?;
    Ok(generated_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_generation() {
        let input = "secure_data";
        let hash = generate_secure_hash(input).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(validate_hash(input, &hash).unwrap());
    }

    #[test]
    fn test_hash_validation() {
        let input = "test_input";
        let hash = generate_secure_hash(input).unwrap();
        assert!(validate_hash(input, &hash).unwrap());
        assert!(!validate_hash("wrong_input", &hash).unwrap());
    }
}