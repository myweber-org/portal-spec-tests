use std::fs::{File, read, write};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;
        
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read source file: {}", e))?;
        
        let encrypted_data = self.xor_transform(&buffer);
        
        let mut dest_file = File::create(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;
        
        dest_file.write_all(&encrypted_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.encrypt_file(source_path, dest_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

pub fn encrypt_string(key: &str, input: &str) -> Vec<u8> {
    let cipher = XorCipher::new(key);
    cipher.xor_transform(input.as_bytes())
}

pub fn decrypt_string(key: &str, encrypted_data: &[u8]) -> String {
    let cipher = XorCipher::new(key);
    let decrypted_bytes = cipher.xor_transform(encrypted_data);
    String::from_utf8_lossy(&decrypted_bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let key = "secret_key";
        let original = "Hello, World!";
        
        let encrypted = encrypt_string(key, original);
        let decrypted = decrypt_string(key, &encrypted);
        
        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key";
        let cipher = XorCipher::new(key);
        
        let original_content = b"Test file content for encryption";
        
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        std::fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        let encrypted_content = std::fs::read(dest_file.path()).unwrap();
        assert_ne!(original_content, encrypted_content.as_slice());
        
        cipher.decrypt_file(dest_file.path(), source_file.path()).unwrap();
        let decrypted_content = std::fs::read(source_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}