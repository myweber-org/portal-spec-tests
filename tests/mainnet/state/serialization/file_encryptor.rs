
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

            let mut processed_buffer = buffer[..bytes_read].to_vec();

            for byte in &mut processed_buffer {
                let key_byte = self.key[key_index % key_len];
                if is_encrypt {
                    *byte ^= key_byte;
                } else {
                    *byte ^= key_byte;
                }
                key_index += 1;
            }

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        self.process_bytes(input.as_bytes(), true)
    }

    pub fn decrypt_string(&self, encrypted: &[u8]) -> String {
        let decrypted = self.process_bytes(encrypted, false);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.key[i % key_len];
            if is_encrypt {
                result.push(byte ^ key_byte);
            } else {
                result.push(byte ^ key_byte);
            }
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
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let encryptor = FileEncryptor::new("test_key");
        
        let mut source_file = NamedTempFile::new()?;
        let dest_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        let test_data = b"Test file content for encryption";
        source_file.write_all(test_data)?;
        source_file.flush()?;
        
        let source_path = source_file.path().to_str().unwrap();
        let dest_path = dest_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        encryptor.encrypt_file(source_path, dest_path)?;
        encryptor.decrypt_file(dest_path, decrypted_path)?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_path)?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(test_data, decrypted_content.as_slice());
        Ok(())
    }
}