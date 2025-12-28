use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    fn next_key_byte(&mut self) -> u8 {
        let byte = self.key[self.key_position];
        self.key_position = (self.key_position + 1) % self.key.len();
        byte
    }

    pub fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| byte ^ self.next_key_byte())
            .collect()
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data = cipher.process_bytes(&buffer);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        let mut cipher = XorCipher::new(key);

        let encrypted = cipher.process_bytes(original_data);
        let mut cipher2 = XorCipher::new(key);
        let decrypted = cipher2.process_bytes(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_password";
        let original_content = b"Sample file content for encryption test.";

        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        let input_path = input_file.path();

        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path();

        encrypt_file(input_path, output_path, key)?;

        let decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        decrypt_file(output_path, decrypted_path, key)?;

        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(original_content.to_vec(), decrypted_content);

        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, ciphertext)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let ciphertext = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, plaintext)
    }
}

pub fn generate_secure_filename(base_name: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    format!("{}_{:x}.enc", base_name, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data that needs protection";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut rng = OsRng;
    let nonce_bytes: [u8; NONCE_SIZE] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte;
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        let password = "strong_password_123";
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let content = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            "correct_password",
        ).unwrap();
        
        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            "wrong_password",
        );
        
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn process_file_encryption(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    
    let file_content = std::fs::read(input_path)?;
    let encrypted = encryptor.encrypt_data(&file_content)?;
    
    std::fs::write(output_path, encrypted)?;
    Ok(())
}