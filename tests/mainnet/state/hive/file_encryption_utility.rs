use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &[u8], salt: &[u8; SALT_LENGTH]) -> Self {
        let mut derived_key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: derived_key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password, salt, params.rounds, &mut derived_key);
        
        FileEncryptor {
            key: Key::<Aes256Gcm>::from_slice(&derived_key).clone(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_content)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_LENGTH));
        
        let encrypted_data = cipher
            .encrypt(nonce, file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = Vec::new();
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&encrypted_data);

        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&output)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_content = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_content)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_content.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_content.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);

        let decrypted_data = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&decrypted_data)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = b"test_password_123";
        let salt = [0xAA; SALT_LENGTH];
        
        let encryptor = FileEncryptor::from_password(password, &salt);
        
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}