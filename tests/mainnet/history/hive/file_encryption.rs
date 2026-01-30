
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

            let processed_data = self.xor_transform(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&mut self, data: &[u8]) -> Vec<u8> {
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original_text = b"Hello, this is a secret message!";
        let key = "my_secret_key_123";
        let mut cipher = XorCipher::new(key);

        let encrypted: Vec<u8> = cipher.xor_transform(original_text);
        cipher.key_position = 0;
        let decrypted = cipher.xor_transform(&encrypted);

        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key";
        let mut cipher = XorCipher::new(key);

        let source_file = NamedTempFile::new()?;
        let dest_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;

        fs::write(source_file.path(), b"Test file content for encryption")?;

        cipher.encrypt_file(source_file.path(), dest_file.path())?;
        cipher.key_position = 0;
        cipher.decrypt_file(dest_file.path(), restored_file.path())?;

        let original_content = fs::read(source_file.path())?;
        let restored_content = fs::read(restored_file.path())?;

        assert_eq!(original_content, restored_content);
        Ok(())
    }
}