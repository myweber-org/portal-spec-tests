
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::error::Error;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, Box<dyn Error>> {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: key.len(),
    };
    
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .map_err(|e| e.into())
        .map(|_| *Key::<Aes256Gcm>::from_slice(&key))
}

pub fn encrypt_data(
    plaintext: &[u8],
    password: &str
) -> Result<EncryptionResult, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|e| e.to_string())?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str
) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    cipher
        .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
        .map_err(|e| e.to_string().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data to protect";
        let password = "StrongPassword123!";
        
        let encrypted = encrypt_data(plaintext, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";
        
        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}
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

    pub fn generate_key(&self) -> Result<Vec<u8>, EncryptionError> {
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                Ok(key.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                Ok(key.to_vec())
            }
        }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                cipher
                    .encrypt(&nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                    .into_iter()
                    .chain(nonce.to_vec())
                    .collect()
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
                cipher
                    .encrypt(&nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                    .into_iter()
                    .chain(nonce.to_vec())
                    .collect()
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }

        let (ciphertext, nonce_slice) = encrypted_data.split_at(encrypted_data.len() - 12);
        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(nonce_slice);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaChaNonce::from_slice(nonce_slice);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = encryptor.generate_key().unwrap();

        let original_content = b"Test data for AES encryption";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = encryptor.generate_key().unwrap();

        let original_content = b"Test data for ChaCha20Poly1305 encryption";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}