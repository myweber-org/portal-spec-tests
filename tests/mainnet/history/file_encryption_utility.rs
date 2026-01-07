use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = match key {
            Some(k) => k.to_vec(),
            None => DEFAULT_KEY.to_vec(),
        };
        FileEncryptor { key }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;
        
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

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, this is a test message for encryption!";
        let encryptor = FileEncryptor::new(None);
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_custom_key() {
        let custom_key = b"my-custom-key-123";
        let encryptor = FileEncryptor::new(Some(custom_key));
        
        let test_data = b"Sensitive information";
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        
        let encrypted_data = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(test_data.as_slice(), encrypted_data.as_slice());
    }
}