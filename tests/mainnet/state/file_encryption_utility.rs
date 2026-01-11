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
    InvalidKeyLength,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
    key: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm, key: &[u8]) -> Result<Self, EncryptionError> {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm if key.len() != 32 => {
                return Err(EncryptionError::InvalidKeyLength)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 if key.len() != 32 => {
                return Err(EncryptionError::InvalidKeyLength)
            }
            _ => {}
        }

        Ok(FileEncryptor {
            algorithm,
            key: key.to_vec(),
        })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&ciphertext)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_aes(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = &ciphertext[0..12];
        let encrypted_data = &ciphertext[12..];
        
        cipher
            .decrypt(Nonce::from_slice(nonce), encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn encrypt_chacha(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher
            .encrypt(ChaChaNonce::from_slice(&nonce), plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_chacha(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }
        
        let key = ChaChaKey::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = &ciphertext[0..12];
        let encrypted_data = &ciphertext[12..];
        
        cipher
            .decrypt(ChaChaNonce::from_slice(nonce), encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let key = [0x42; 32];
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm, &key).unwrap();
        
        let plaintext = b"Hello, World! This is a test message.";
        
        let ciphertext = encryptor.encrypt_aes(plaintext).unwrap();
        let decrypted = encryptor.decrypt_aes(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let key = [0x42; 32];
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305, &key).unwrap();
        
        let plaintext = b"Hello, World! This is a test message.";
        
        let ciphertext = encryptor.encrypt_chacha(plaintext).unwrap();
        let decrypted = encryptor.decrypt_chacha(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_file_operations() {
        let key = [0x42; 32];
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm, &key).unwrap();
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Test file content for encryption";
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data, decrypted_data.as_slice());
    }
}