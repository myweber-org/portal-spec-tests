use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());
    
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < NONCE_SIZE {
        return Err("Ciphertext too short".into());
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, encrypted_data)?;
    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    OsRng.gen()
}