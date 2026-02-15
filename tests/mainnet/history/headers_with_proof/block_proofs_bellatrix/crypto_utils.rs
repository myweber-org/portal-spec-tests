
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_token(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_api_key() -> String {
    generate_secure_token(32)
}

pub fn generate_session_token() -> String {
    generate_secure_token(64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token(16);
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_session_token() {
        let token = generate_session_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_tokens_are_unique() {
        let token1 = generate_secure_token(16);
        let token2 = generate_secure_token(16);
        assert_ne!(token1, token2);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = Nonce::from_slice(&generate_nonce());

    cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let data = &ciphertext[NONCE_SIZE..];

    cipher
        .decrypt(nonce, data)
        .context("Decryption failed")
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let key = [0x42; 32];
        let plaintext = b"Test data";
        
        let mut encrypted = encrypt_data(plaintext, &key).unwrap();
        encrypted[20] ^= 0xFF;
        
        assert!(decrypt_data(&encrypted, &key).is_err());
    }
}use rand::Rng;
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;

pub fn generate_salt() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..SALT_LENGTH).map(|_| rng.gen()).collect()
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

pub fn verify_password(password: &str, salt: &[u8], expected_hash: &[u8]) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == expected_hash
}
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_token(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> String {
    let token: String = generate_token(32);
    format!("sk_{}", token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_token(16);
        assert_eq!(token.len(), 16);
    }

    #[test]
    fn test_secure_token_format() {
        let token = generate_secure_token();
        assert!(token.starts_with("sk_"));
        assert_eq!(token.len(), 35);
    }
}