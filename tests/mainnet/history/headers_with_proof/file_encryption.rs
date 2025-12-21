use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());
    
    let mut ciphertext = cipher.encrypt(nonce, plaintext)?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce);
    result.append(&mut ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < NONCE_SIZE {
        return Err("Ciphertext too short".into());
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let encrypted_data = &ciphertext[NONCE_SIZE..];
    
    cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let plaintext = b"Secret data that needs protection";
        
        let encrypted = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}