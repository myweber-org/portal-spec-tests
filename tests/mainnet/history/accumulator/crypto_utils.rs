use sha2::{Digest, Sha256};
use rand::{RngCore, thread_rng};

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut buffer = vec![0u8; len];
    rng.fill_bytes(&mut buffer);
    buffer
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn sha256_hex_string(data: &[u8]) -> String {
    let hash = sha256_hash(data);
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_sha256_consistent() {
        let data = b"hello world";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_format() {
        let data = b"test";
        let hex = sha256_hex_string(data);
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};
use std::iter;

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = thread_rng();
    let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    
    iter::repeat_with(one_char).take(length).collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn generate_salt() -> String {
    generate_random_string(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_string_length() {
        let random = generate_random_string(16);
        assert_eq!(random.len(), 16);
    }

    #[test]
    fn test_hash_consistency() {
        let password = "secure_password";
        let salt = "random_salt";
        
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_salt_generation() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
        assert_ne!(salt1, salt2);
    }
}