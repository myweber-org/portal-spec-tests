use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let (ciphertext, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::generate(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::generate(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, file_data.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, nonce.to_vec())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::generate(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::generate(&mut OsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, file_data.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, nonce.to_vec())
            }
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        output_file
            .write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output_file
            .write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let (nonce, ciphertext) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                if encrypted_data.len() < 12 {
                    return Err("Invalid encrypted data format".to_string());
                }
                let (n, ct) = encrypted_data.split_at(12);
                (n.to_vec(), ct.to_vec())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                if encrypted_data.len() < 12 {
                    return Err("Invalid encrypted data format".to_string());
                }
                let (n, ct) = encrypted_data.split_at(12);
                (n.to_vec(), ct.to_vec())
            }
        };

        let plaintext = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::generate(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(&nonce);
                cipher
                    .decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::generate(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(&nonce);
                cipher
                    .decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
        };

        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let test_data = b"Test encryption data";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another test for ChaCha";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}