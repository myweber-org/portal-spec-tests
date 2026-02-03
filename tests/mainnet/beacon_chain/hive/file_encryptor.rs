
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_cipher(&buffer);

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }

    fn xor_cipher(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let test_content = b"Hello, this is a test message for encryption!";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), test_content).unwrap();

        encryptor
            .encrypt_file(source_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let encryptor = FileEncryptor::new("");
        let data = b"test data";
        let encrypted = encryptor.xor_cipher(data);
        assert_eq!(data, encrypted.as_slice());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, is_encrypt: bool) -> io::Result<()> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file not found: {}", source_path),
            ));
        }

        let mut source_file = fs::File::open(source)?;
        let mut dest_file = fs::File::create(dest)?;

        let mut buffer = [0; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            
            for byte in processed_buffer.iter_mut() {
                let key_byte = self.key[key_index];
                *byte = if is_encrypt {
                    *byte ^ key_byte
                } else {
                    *byte ^ key_byte
                };
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_bytes(text.as_bytes(), true)
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_bytes(data, false);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.key[i % key_len];
            result.push(if is_encrypt {
                byte ^ key_byte
            } else {
                byte ^ key_byte
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_string() {
        let encryptor = FileEncryptor::new("secret_key");
        let original = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_ne!(encrypted, original.as_bytes());
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new("test_key");
        let original_content = b"Test file content for encryption";
        
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        encryptor
            .encrypt_file(
                source_file.path().to_str().unwrap(),
                dest_file.path().to_str().unwrap(),
            )
            .unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        let decrypt_file = NamedTempFile::new().unwrap();
        encryptor
            .decrypt_file(
                dest_file.path().to_str().unwrap(),
                decrypt_file.path().to_str().unwrap(),
            )
            .unwrap();
        
        let decrypted_content = fs::read(decrypt_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        let test_key = Some(0xAA);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Testing with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }
}use aes_gcm::{
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
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
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
        let password = "secure_password_123";
        let encryptor = FileEncryptor::from_password(password).unwrap();
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}