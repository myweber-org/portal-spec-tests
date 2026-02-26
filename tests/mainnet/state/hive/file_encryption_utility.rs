use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = nonce.to_vec();
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("test_password").unwrap();
        let test_data = b"Secret file content for testing";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(FileEncryptor { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&rand::random::<[u8; NONCE_SIZE]>());
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    rand::random::<[u8; 32]>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
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
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub struct FileEncryptor {
    key: u8,
}

impl FileEncryptor {
    pub fn new(key: Option<u8>) -> Self {
        FileEncryptor {
            key: key.unwrap_or(DEFAULT_KEY),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0u8; 1024];
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let encrypted_chunk: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| byte ^ self.key)
                .collect();

            dest_file.write_all(&encrypted_chunk)?;
        }

        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }
}

pub fn process_files(
    encryptor: &FileEncryptor,
    input_dir: &Path,
    output_dir: &Path,
    operation: OperationType,
) -> io::Result<Vec<String>> {
    let mut processed_files = Vec::new();

    if input_dir.is_dir() {
        for entry in fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let output_path = output_dir.join(format!(
                    "{}_{}",
                    file_name,
                    match operation {
                        OperationType::Encrypt => "encrypted",
                        OperationType::Decrypt => "decrypted",
                    }
                ));

                match operation {
                    OperationType::Encrypt => encryptor.encrypt_file(&path, &output_path)?,
                    OperationType::Decrypt => encryptor.decrypt_file(&path, &output_path)?,
                }

                processed_files.push(output_path.to_string_lossy().into_owned());
            }
        }
    }

    Ok(processed_files)
}

pub enum OperationType {
    Encrypt,
    Decrypt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_xor_encryption() {
        let encryptor = FileEncryptor::new(Some(0x55));
        let test_data = b"Hello, World!";
        let encrypted: Vec<u8> = test_data.iter().map(|&b| b ^ 0x55).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|&b| b ^ 0x55).collect();

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_roundtrip() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let original_path = temp_dir.path().join("original.txt");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.txt");

        fs::write(&original_path, "Test data for encryption")?;

        let encryptor = FileEncryptor::new(None);
        encryptor.encrypt_file(&original_path, &encrypted_path)?;
        encryptor.decrypt_file(&encrypted_path, &decrypted_path)?;

        let original_content = fs::read(&original_path)?;
        let decrypted_content = fs::read(&decrypted_path)?;

        assert_eq!(original_content, decrypted_content);
        Ok(())
    }
}