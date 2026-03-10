use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};

const NONCE_SIZE: usize = 12;

pub fn generate_key() -> Result<Vec<u8>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    Ok(key.to_vec())
}

pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    cipher.encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("Encryption failed: {}", e))
}

pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key().unwrap();
        let message = b"Secret message for encryption test";
        
        let ciphertext = encrypt(message, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(message.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();
        let message = b"Test message";
        
        let ciphertext = encrypt(message, &key1).unwrap();
        let result = decrypt(&ciphertext, &key2);
        
        assert!(result.is_err());
    }
}
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
    fn test_generate_random_string() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
        
        let another_result = generate_random_string(10);
        assert_ne!(result, another_result);
    }

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> String {
    let mut rng = thread_rng();
    let token: String = (0..32)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    token
}

pub fn generate_numeric_code(digits: u32) -> String {
    let mut rng = thread_rng();
    let max = 10u32.pow(digits);
    let code = rng.gen_range(0..max);
    format!("{:0width$}", code, width = digits as usize)
}