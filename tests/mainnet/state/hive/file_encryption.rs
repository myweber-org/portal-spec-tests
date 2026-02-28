
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use rand_core::RngCore;
use std::error::Error;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8; SALT_LENGTH]) -> Result<Key<Aes256Gcm>, Box<dyn Error>> {
    let salt_string = SaltString::encode_b64(salt)?;
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt_string)?;
    
    let hash_bytes = password_hash.hash.ok_or("Failed to generate hash")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32].try_into()?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(plaintext: &[u8], password: &str) -> Result<EncryptedData, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptedData {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_data(encrypted: &EncryptedData, password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Secret message for encryption test";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let original_data = b"Another secret message";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}