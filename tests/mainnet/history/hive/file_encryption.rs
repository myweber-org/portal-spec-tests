use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    cipher_type: CipherType,
}

impl FileEncryptor {
    pub fn new(cipher_type: CipherType) -> Self {
        Self { cipher_type }
    }

    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.cipher_type {
            CipherType::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::generate(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)?;
                let mut result = nonce.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
            CipherType::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)?;
                let mut result = nonce.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.cipher_type {
            CipherType::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
            CipherType::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption() {
        let encryptor = FileEncryptor::new(CipherType::Aes256Gcm);
        let key = [0u8; 32];
        let plaintext = b"secret message";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption() {
        let encryptor = FileEncryptor::new(CipherType::ChaCha20Poly1305);
        let key = [0u8; 32];
        let plaintext = b"another secret";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}