
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    let mut buffer = Vec::new();
    
    let mut input_file = fs::File::open(input_path)?;
    input_file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key_bytes);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn validate_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 256
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![1, 2, 3, 4, 5];
        let original = data.clone();
        let key = b"secret";
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), b"test data").unwrap();
        
        let result = process_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            "mykey123"
        );
        
        assert!(result.is_ok());
        
        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted, b"test data");
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid"));
        assert!(!validate_key(""));
        assert!(validate_key(&"a".repeat(256)));
        assert!(!validate_key(&"a".repeat(257)));
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

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path, is_encrypt: bool) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_buffer: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| {
                    let result = byte ^ self.key[key_index];
                    key_index = (key_index + 1) % key_len;
                    result
                })
                .collect();

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        self.process_bytes(text.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_bytes(data);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original_text = "Hello, World! This is a test message.";

        let encrypted = encryptor.encrypt_string(original_text);
        let decrypted = encryptor.decrypt_string(&encrypted);

        assert_ne!(encrypted, original_text.as_bytes());
        assert_eq!(decrypted, original_text);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let encryptor = FileEncryptor::new("file_encryption_key");

        let mut source_file = NamedTempFile::new()?;
        let test_content = b"Sample file content for encryption test";
        source_file.write_all(test_content)?;

        let encrypted_file = NamedTempFile::new()?;
        encryptor.encrypt_file(source_file.path(), encrypted_file.path())?;

        let decrypted_file = NamedTempFile::new()?;
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, test_content);

        Ok(())
    }

    #[test]
    fn test_symmetry() {
        let encryptor = FileEncryptor::new("test_key");
        let data = [0u8, 255u8, 128u8, 64u8, 32u8];

        let encrypted = encryptor.process_bytes(&data);
        let decrypted = encryptor.process_bytes(&encrypted);

        assert_eq!(decrypted, data);
    }
}