use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidFileSize,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let metadata = file.metadata()?;
        
        if metadata.len() > 10 * 1024 * 1024 * 1024 {
            return Err(EncryptionError::InvalidFileSize);
        }

        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&ciphertext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map(|mut ciphertext| {
                ciphertext.splice(0..0, nonce.iter().cloned());
                ciphertext
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&ciphertext[0..12]);
        let actual_ciphertext = &ciphertext[12..];

        cipher
            .decrypt(nonce, actual_ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let nonce = ChaChaNonce::from_slice(&nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map(|mut ciphertext| {
                ciphertext.splice(0..0, nonce.iter().cloned());
                ciphertext
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = ChaChaNonce::from_slice(&ciphertext[0..12]);
        let actual_ciphertext = &ciphertext[12..];

        cipher
            .decrypt(nonce, actual_ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = FileEncryptor::generate_key();
        
        let plaintext = b"Hello, World! This is a test message.";
        
        let ciphertext = encryptor.encrypt_aes(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt_aes(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = FileEncryptor::generate_key();
        
        let plaintext = b"Another test with different algorithm.";
        
        let ciphertext = encryptor.encrypt_chacha(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt_chacha(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = FileEncryptor::generate_key();
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test data";
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor
            .encrypt_file(input_file.path(), output_file.path(), &key)
            .unwrap();
        
        encryptor
            .decrypt_file(output_file.path(), decrypted_file.path(), &key)
            .unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}