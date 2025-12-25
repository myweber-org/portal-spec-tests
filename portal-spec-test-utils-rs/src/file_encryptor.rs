
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
                "Source file does not exist",
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

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        self.process_bytes(input.as_bytes())
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted = self.process_bytes(data);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            result.push(byte ^ self.key[i % key_len]);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new("test_key_123");
        let original_content = b"This is a test file content for encryption.";
        
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        encryptor
            .encrypt_file(
                source_file.path().to_str().unwrap(),
                dest_file.path().to_str().unwrap(),
            )
            .unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(original_content, encrypted_content.as_slice());
        
        let decrypt_file = NamedTempFile::new().unwrap();
        encryptor
            .decrypt_file(
                dest_file.path().to_str().unwrap(),
                decrypt_file.path().to_str().unwrap(),
            )
            .unwrap();
        
        let decrypted_content = fs::read(decrypt_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}