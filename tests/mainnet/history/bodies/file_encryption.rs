
use std::fs::{self, File};
use std::io::{Read, Write};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), std::io::Error> {
        let mut input_file = File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.process_bytes(&buffer);

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), std::io::Error> {
        self.encrypt_file(input_path, output_path)
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_length = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_length])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XORCipher::new("secret_key");
        let test_data = b"Hello, World!";
        
        let encrypted = cipher.process_bytes(test_data);
        assert_ne!(encrypted, test_data);
        
        let decrypted = cipher.process_bytes(&encrypted);
        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_key");
        let original_content = b"Sample file content for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        let encrypted_content = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        let decrypt_file = NamedTempFile::new().unwrap();
        cipher.decrypt_file(output_file.path(), decrypt_file.path()).unwrap();
        let decrypted_content = fs::read(decrypt_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}