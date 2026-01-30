
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }
        
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let ciphertext_data = &ciphertext[12..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext_data)?;
        Ok(plaintext)
    }
}

pub fn process_encryption() -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new()?;
    
    let test_data = b"Sensitive information that needs protection";
    println!("Original data: {:?}", String::from_utf8_lossy(test_data));
    
    let encrypted = encryptor.encrypt_data(test_data)?;
    println!("Encrypted data length: {} bytes", encrypted.len());
    
    let decrypted = encryptor.decrypt_data(&encrypted)?;
    println!("Decrypted data: {:?}", String::from_utf8_lossy(&decrypted));
    
    assert_eq!(test_data, decrypted.as_slice());
    println!("Encryption/decryption successful!");
    
    Ok(())
}