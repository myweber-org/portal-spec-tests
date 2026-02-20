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

        let mut buffer = [0; 4096];
        self.key_position = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_data = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection!";
        let key = "strong_password_123";

        let mut cipher = XorCipher::new(key);

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), original_content).unwrap();

        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        
        let mut cipher2 = XorCipher::new(key);
        cipher2.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }

    #[test]
    fn test_different_keys_produce_different_results() {
        let content = b"Test data";
        let key1 = "key1";
        let key2 = "key2";

        let mut cipher1 = XorCipher::new(key1);
        let mut cipher2 = XorCipher::new(key2);

        let source_file = NamedTempFile::new().unwrap();
        let encrypted1 = NamedTempFile::new().unwrap();
        let encrypted2 = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), content).unwrap();

        cipher1.encrypt_file(source_file.path(), encrypted1.path()).unwrap();
        cipher2.encrypt_file(source_file.path(), encrypted2.path()).unwrap();

        let result1 = fs::read(encrypted1.path()).unwrap();
        let result2 = fs::read(encrypted2.path()).unwrap();

        assert_ne!(result1, result2);
    }
}