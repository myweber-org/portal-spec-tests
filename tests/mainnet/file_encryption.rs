
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::generate(&mut OsRng);
                let ciphertext = cipher.encrypt(&nonce, plaintext)?;
                let mut result = nonce.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
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
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                if ciphertext.len() < 12 {
                    return Err("Invalid ciphertext length".into());
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                if ciphertext.len() < 12 {
                    return Err("Invalid ciphertext length".into());
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, encrypted_data).map_err(|e| e.into())
            }
        }
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let plaintext = b"Secret message for AES encryption";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let plaintext = b"Secret message for ChaCha encryption";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        let plaintext = b"Test message";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let result = encryptor.decrypt(&ciphertext, &wrong_key);
        
        assert!(result.is_err());
    }
}