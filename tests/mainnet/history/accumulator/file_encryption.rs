
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    let key_len = key.len();
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key_len];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; BUFFER_SIZE];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let data_slice = &mut buffer[..bytes_read];
        xor_cipher(data_slice, key);
        
        output_file.write_all(data_slice)?;
    }
    
    output_file.flush()?;
    Ok(())
}

pub fn validate_key(key: &str) -> Result<Vec<u8>, &'static str> {
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    
    if key.len() < 8 {
        return Err("Key must be at least 8 characters long");
    }
    
    Ok(key.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let key = b"secretkey";
        
        let mut encrypted = original.to_vec();
        xor_cipher(&mut encrypted, key);
        
        assert_ne!(encrypted, original);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, key);
        
        assert_eq!(decrypted, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        let key = b"testkey123";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, key);
        
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
    
    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("validkey123").is_ok());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
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
        Self { cipher }
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

pub fn generate_secure_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_cycle() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Confidential data for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
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
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

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
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())?;
        
        let mut output = File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        
        let mut output = File::create(output_path)?;
        output.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn encrypt_directory(password: &str, dir_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let encryptor = FileEncryptor::new(password)?;
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(path.file_name().unwrap());
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}