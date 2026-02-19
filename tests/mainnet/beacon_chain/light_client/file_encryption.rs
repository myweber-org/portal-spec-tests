
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
    let data = std::fs::read(input_path)?;
    
    let encrypted = encryptor.encrypt_data(&data)?;
    std::fs::write(output_path, encrypted)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret message for encryption test";
        
        let encrypted = encryptor.encrypt_data(test_data).unwrap();
        let decrypted = encryptor.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        let test_content = b"Test file content for encryption";
        std::fs::write(input_file.path(), test_content).unwrap();
        
        process_file_encryption(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let encrypted_data = std::fs::read(output_file.path()).unwrap();
        assert_ne!(test_content, encrypted_data.as_slice());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test file.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Testing with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
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
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        let original_content = b"Sample file content for encryption test";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;
        assert_ne!(encrypted_content.as_slice(), original_content);
        
        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        assert_eq!(decrypted_content.as_slice(), original_content);
        
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
        
        let key = Key::<Aes256Gcm>::from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = nonce.to_vec();
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        if data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_secure_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok((ciphertext, key.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"secret message";
        let (ciphertext, key) = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&ciphertext, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn encrypt_file(source_path: &str, dest_path: &str, key: &[u8]) -> io::Result<()> {
    process_file(source_path, dest_path, key)
}

pub fn decrypt_file(source_path: &str, dest_path: &str, key: &[u8]) -> io::Result<()> {
    process_file(source_path, dest_path, key)
}

fn process_file(source_path: &str, dest_path: &str, key: &[u8]) -> io::Result<()> {
    let source = Path::new(source_path);
    let dest = Path::new(dest_path);

    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source file '{}' not found", source_path),
        ));
    }

    let mut source_file = fs::File::open(source)?;
    let mut dest_file = fs::File::create(dest)?;

    let mut buffer = [0u8; BUFFER_SIZE];
    let key_len = key.len();
    let mut key_index = 0;

    loop {
        let bytes_read = source_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for i in 0..bytes_read {
            buffer[i] ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }

        dest_file.write_all(&buffer[..bytes_read])?;
    }

    dest_file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt() {
        let key = b"secret_key";
        let original_data = b"This is a test message for encryption and decryption.";

        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_data).unwrap();
        let source_path = source_file.path().to_str().unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let encrypted_path = encrypted_file.path().to_str().unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();

        encrypt_file(source_path, encrypted_path, key).unwrap();
        decrypt_file(encrypted_path, decrypted_path, key).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}