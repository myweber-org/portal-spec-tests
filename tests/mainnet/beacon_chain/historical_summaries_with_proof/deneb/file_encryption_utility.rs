use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(&encrypted_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;
        Ok(())
    }

    fn aes_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Nonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
    }

    fn aes_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher_key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
            .map(|mut ciphertext| {
                let mut result = nonce.to_vec();
                result.append(&mut ciphertext);
                result
            })
    }

    fn chacha_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);
        let cipher_key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let test_data = b"Test encryption data";
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        let encrypted = encryptor.aes_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.aes_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let test_data = b"Test encryption data";
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let encrypted = encryptor.chacha_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.chacha_decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test content";
        fs::write(input_file.path(), test_data).unwrap();
        
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        encryptor.encrypt_file(input_file.path(), output_file.path(), &key).unwrap();
        encryptor.decrypt_file(output_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}