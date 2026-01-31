
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption() {
        let key = b"secret_key";
        let test_data = b"Hello, Rust encryption!";

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let key = b"secret";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let content = b"Test file content for encryption";
        let key = "test_key_123";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), content)?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, content);
        
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        let decrypted = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted, content);
        
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key.as_bytes());
        assert_ne!(data.as_slice(), original);
        
        xor_cipher(&mut data, key.as_bytes());
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let original_content = b"Confidential data that needs protection";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;
        assert_ne!(encrypted_content, original_content);
        
        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        assert_eq!(decrypted_content, original_content);
        
        Ok(())
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

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher.encrypt(&nonce, data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = nonce.to_vec();
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, output)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data for encryption test";
        let password = "strong_password_123";
        
        let encryptor = FileEncryptor::new(password).unwrap();
        
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