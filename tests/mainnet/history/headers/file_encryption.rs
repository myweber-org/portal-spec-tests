
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }

    let (key_bytes, ciphertext) = content.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, secure world!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn process_files() -> Result<(), Box<dyn std::error::Error>> {
    let cipher = XorCipher::new("secret_key_123");
    
    let test_data = b"Hello, this is a test message!";
    let encrypted = cipher.xor_transform(test_data);
    let decrypted = cipher.xor_transform(&encrypted);
    
    assert_eq!(test_data.to_vec(), decrypted);
    
    println!("Encryption test passed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("test_key");
        let original = b"Test data for encryption";
        let encrypted = cipher.xor_transform(original);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_ne!(original, encrypted.as_slice());
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XorCipher::new("file_test_key");
        
        let mut input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        let test_content = b"File encryption test content";
        input_file.write_all(test_content)?;
        
        cipher.encrypt_file(input_file.path(), output_file.path())?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(test_content, encrypted_content.as_slice());
        
        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(test_content, decrypted_content.as_slice());
        
        Ok(())
    }
}