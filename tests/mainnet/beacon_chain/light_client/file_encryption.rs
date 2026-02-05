
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn process_file_encryption(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let data = std::fs::read(input_path)?;
    
    let encrypted = encryptor.encrypt_data(&data)?;
    std::fs::write(output_path, encrypted)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret message for encryption test";
        
        let encrypted = encryptor.encrypt_data(test_data).unwrap();
        let decrypted = encryptor.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        let test_content = b"Test file content for encryption";
        std::fs::write(input_file.path(), test_content).unwrap();
        
        process_file_encryption(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let encrypted_data = std::fs::read(output_file.path()).unwrap();
        assert_ne!(test_content, encrypted_data.as_slice());
    }
}