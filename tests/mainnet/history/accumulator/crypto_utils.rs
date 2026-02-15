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
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .context("Encryption failed")?;
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_slice, encrypted_data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_slice);
    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_encryption_roundtrip() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let plaintext = b"Secret message for encryption test";
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_decryption_failure() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let invalid_ciphertext = b"tooshort";
        let result = decrypt_data(invalid_ciphertext, &key);
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .context("Encryption failed")?;
    
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let plaintext = b"Secret message for testing";
        
        let ciphertext = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_decryption() {
        let key = generate_key();
        let invalid_data = b"Too short";
        
        let result = decrypt_data(invalid_data, &key);
        assert!(result.is_err());
    }
}
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(16);
        let s2 = generate_random_string(16);
        assert_eq!(s1.len(), 16);
        assert_eq!(s2.len(), 16);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_hash_password() {
        let hash1 = hash_password("mypassword", "somesalt");
        let hash2 = hash_password("mypassword", "somesalt");
        let hash3 = hash_password("otherpassword", "somesalt");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }
}