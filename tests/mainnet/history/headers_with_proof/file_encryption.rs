use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            output_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    fn process_data(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            result.push(byte ^ self.key[key_index]);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, World! This is a test message.";

        let encrypted = cipher.encrypt_data(original_data);
        assert_ne!(encrypted, original_data);

        let decrypted = cipher.decrypt_data(&encrypted);
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("file_encryption_key");
        
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Sample file content for encryption testing";
        input_file.write_all(test_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(input_file.path(), output_file.path())?;
        
        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(decrypted_content, test_data);
        Ok(())
    }
}