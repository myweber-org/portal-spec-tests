use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = key.unwrap_or(DEFAULT_KEY).to_vec();
        FileCipher { key }
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

        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();

            for byte in processed_buffer.iter_mut() {
                *byte ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&processed_buffer)?;
        }

        if is_encrypt {
            println!("File encrypted successfully: {} -> {}", source_path, dest_path);
        } else {
            println!("File decrypted successfully: {} -> {}", source_path, dest_path);
        }

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_text(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_text(data);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_text(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            result.push(byte ^ self.key[i % key_len]);
        }

        result
    }
}

pub fn validate_key(key: &[u8]) -> bool {
    !key.is_empty() && key.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_string() {
        let cipher = FileCipher::new(Some(b"test-key-123"));
        let original = "Hello, World! This is a secret message.";
        
        let encrypted = cipher.encrypt_string(original);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted);
    }

    #[test]
    fn test_file_operations() {
        let cipher = FileCipher::new(Some(b"file-test-key"));
        let original_content = b"Test file content for encryption demonstration.";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(
            source_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        cipher.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key(b"valid-key-123"));
        assert!(validate_key(b"12345678"));
        assert!(!validate_key(b"short"));
        assert!(!validate_key(b""));
    }
}