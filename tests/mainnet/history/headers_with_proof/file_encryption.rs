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

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;
        
        let mut buffer = [0u8; 4096];
        
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            let processed_data = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }
        
        self.reset_key_position();
        Ok(())
    }

    fn process_chunk(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    fn reset_key_position(&mut self) {
        self.key_position = 0;
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters long".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let key = "strong_secret_key_123!";
        let original_text = b"Hello, this is a secret message!";
        
        let mut cipher = XorCipher::new(key);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_text).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(temp_file.path(), encrypted_file.path()).unwrap();
        cipher.reset_key_position();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid_long_key").is_ok());
        assert!(validate_key("short").is_err());
        assert!(validate_key("").is_err());
    }
}