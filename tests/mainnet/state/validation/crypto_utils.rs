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