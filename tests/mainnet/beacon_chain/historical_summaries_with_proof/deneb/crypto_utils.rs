
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    generate_random_string(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string_length() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_generate_secure_token_length() {
        let result = generate_secure_token();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_random_strings_different() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        assert_ne!(s1, s2);
    }
}use sha2::{Digest, Sha256};
use rand::{thread_rng, RngCore};

/// Generates a cryptographically secure random byte array of the specified length.
pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Computes the SHA-256 hash of the input data and returns it as a hex string.
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = sha256_hash(data);
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash, expected);
    }
}