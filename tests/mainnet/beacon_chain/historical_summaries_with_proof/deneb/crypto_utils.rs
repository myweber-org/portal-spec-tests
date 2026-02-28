use sha2::{Digest, Sha256};
use std::error::Error;

pub fn compute_sha256(data: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_sha256(data: &[u8], expected_hash: &str) -> Result<bool, Box<dyn Error>> {
    let computed_hash = compute_sha256(data)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hashing() {
        let data = b"hello world";
        let hash = compute_sha256(data).unwrap();
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash, expected);
        assert!(verify_sha256(data, expected).unwrap());
    }

    #[test]
    fn test_sha256_empty() {
        let data = b"";
        let hash = compute_sha256(data).unwrap();
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash, expected);
    }
}use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 32;

pub fn generate_salt() -> String {
    let mut rng = thread_rng();
    (0..SALT_LENGTH)
        .map(|_| rng.gen_range(33..127) as u8 as char)
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_api_token() -> String {
    let mut rng = thread_rng();
    (0..TOKEN_LENGTH)
        .map(|_| rng.gen_range(48..123) as u8 as char)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), SALT_LENGTH);
    }

    #[test]
    fn test_hash_consistency() {
        let password = "secure_password";
        let salt = "test_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_token_alphanumeric() {
        let token = generate_api_token();
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(token.len(), TOKEN_LENGTH);
    }
}