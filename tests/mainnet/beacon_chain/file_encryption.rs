
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
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
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&plaintext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&plaintext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let (nonce, ciphertext) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let nonce_len = 12;
                if data.len() < nonce_len {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
                }
                (&data[..nonce_len], &data[nonce_len..])
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let nonce_len = 12;
                if data.len() < nonce_len {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
                }
                (&data[..nonce_len], &data[nonce_len..])
            }
        };

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(ciphertext, key, nonce)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(ciphertext, key, nonce)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn aes_encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("AES-256-GCM requires 32-byte key".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce_bytes: [u8; 12] = OsRng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn aes_decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("AES-256-GCM requires 32-byte key".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok(plaintext)
    }

    fn chacha_encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("ChaCha20Poly1305 requires 32-byte key".to_string()));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce_bytes: [u8; 12] = OsRng.gen();
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn chacha_decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("ChaCha20Poly1305 requires 32-byte key".to_string()));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce = ChaChaNonce::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;

        Ok(plaintext)
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output = key.to_vec();
    output.extend_from_slice(nonce);
    output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, output)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted = fs::read(input_path)?;
    
    if encrypted.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&encrypted[0..32]);
    let nonce = Nonce::from_slice(&encrypted[32..44]);
    let ciphertext = &encrypted[44..];
    
    let cipher = Aes256Gcm::new(key);
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data that needs protection";
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(plaintext_file.path(), test_data).unwrap();
        
        encrypt_file(
            plaintext_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted, test_data);
    }
}