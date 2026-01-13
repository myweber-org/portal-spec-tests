use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
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
        self.key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[self.key_index];
                self.key_index = (self.key_index + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        self.key_index = 0;

        for &byte in data {
            result.push(byte ^ self.key[self.key_index]);
            self.key_index = (self.key_index + 1) % self.key.len();
        }

        result
    }

    pub fn decrypt_data(&mut self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

pub fn validate_key(key: &str) -> bool {
    !key.is_empty() && key.len() >= 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut cipher = XorCipher::new("secret_key_123");
        let original_data = b"Hello, this is a secret message!";
        
        let encrypted = cipher.encrypt_data(original_data);
        cipher.key_index = 0;
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let mut cipher = XorCipher::new("test_password");
        let test_content = b"Sample file content for encryption test";
        
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), test_content).unwrap();
        
        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(test_content, encrypted_content.as_slice());
        
        cipher.key_index = 0;
        cipher.decrypt_file(dest_file.path(), source_file.path()).unwrap();
        
        let decrypted_content = fs::read(source_file.path()).unwrap();
        assert_eq!(test_content, decrypted_content.as_slice());
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid_key"));
        assert!(validate_key("12345678"));
        assert!(!validate_key(""));
        assert!(!validate_key("abc"));
    }
}