use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::RngCore;
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;

pub struct FileEncryptor {
    password: String,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = self.read_file(input_path)?;
        
        let mut salt = [0u8; SALT_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        
        let key = self.derive_key(&salt);
        
        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut iv);
        
        let cipher = Aes256Cbc::new_from_slices(&key, &iv)
            .map_err(|e| format!("Cipher initialization failed: {}", e))?;
        
        let ciphertext = cipher.encrypt_vec(&mut file_data);
        
        let mut output_data = Vec::new();
        output_data.extend_from_slice(&salt);
        output_data.extend_from_slice(&iv);
        output_data.extend_from_slice(&ciphertext);
        
        self.write_file(output_path, &output_data)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let encrypted_data = self.read_file(input_path)?;
        
        if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
            return Err("File too short to contain valid encrypted data".to_string());
        }
        
        let salt = &encrypted_data[0..SALT_LENGTH];
        let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
        let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];
        
        let key = self.derive_key(salt);
        
        let cipher = Aes256Cbc::new_from_slices(&key, iv)
            .map_err(|e| format!("Cipher initialization failed: {}", e))?;
        
        let mut buffer = ciphertext.to_vec();
        let decrypted_data = cipher.decrypt_vec(&mut buffer)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        self.write_file(output_path, &decrypted_data)
    }

    fn derive_key(&self, salt: &[u8]) -> [u8; KEY_LENGTH] {
        let mut hasher = Sha256::new();
        hasher.update(&self.password.as_bytes());
        hasher.update(salt);
        
        let mut key = [0u8; KEY_LENGTH];
        key.copy_from_slice(&hasher.finalize());
        key
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        let mut file = fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        Ok(buffer)
    }

    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), String> {
        let mut file = fs::File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        file.write_all(data)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password);
        
        let original_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}