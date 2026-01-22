
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
        self.key_position = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[self.key_position];
                self.key_position = (self.key_position + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        self.key_position = 0;

        for &byte in data {
            result.push(byte ^ self.key[self.key_position]);
            self.key_position = (self.key_position + 1) % self.key.len();
        }

        result
    }

    pub fn decrypt_data(&mut self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

pub fn calculate_file_hash(path: &Path) -> io::Result<u32> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 4096];
    let mut hash: u32 = 0x811c9dc5;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for &byte in &buffer[..bytes_read] {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";

        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.encrypt_data(original_data);

        let mut cipher2 = XorCipher::new(key);
        let decrypted = cipher2.decrypt_data(&encrypted);

        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let original_content = b"Sample file content for encryption test.";

        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(original_content)?;
        let source_path = source_file.path();

        let mut dest_file = NamedTempFile::new()?;
        let dest_path = dest_file.path();

        let mut cipher = XorCipher::new(key);
        cipher.encrypt_file(source_path, dest_path)?;

        let mut cipher2 = XorCipher::new(key);
        let mut decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        cipher2.decrypt_file(dest_path, decrypted_path)?;

        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }

    #[test]
    fn test_hash_calculation() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test content for hash")?;

        let hash = calculate_file_hash(temp_file.path())?;
        assert_ne!(hash, 0);

        Ok(())
    }
}