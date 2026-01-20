use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption";
        let (ciphertext, nonce) = encrypt_data(original_data).unwrap();
        
        let key = Aes256Gcm::generate_key(&mut OsRng).to_vec();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let encrypted = cipher.encrypt(Nonce::from_slice(&nonce), original_data).unwrap();
        
        assert_eq!(encrypted, ciphertext);
        
        let decrypted = decrypt_data(&ciphertext, &nonce, &key).unwrap();
        assert_eq!(decrypted, original_data);
    }
}