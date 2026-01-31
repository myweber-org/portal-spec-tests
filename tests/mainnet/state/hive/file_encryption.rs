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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> Result<(), String> {
        let input_path_obj = Path::new(input_path);
        let output_path_obj = Path::new(output_path);

        if !input_path_obj.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_process(&buffer);

        let mut output_file = File::create(output_path_obj)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_process(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let cipher = XORCipher::new(key);
        
        let original_data = b"Hello, World! This is a test message.";
        let encrypted = cipher.xor_process(original_data);
        let decrypted = cipher.xor_process(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key_123";
        let cipher = XORCipher::new(key);
        
        let mut input_file = NamedTempFile::new().unwrap();
        let test_data = b"Sample data for encryption test";
        input_file.write_all(test_data).unwrap();
        
        let input_path = input_file.path().to_str().unwrap();
        let encrypted_path = "test_encrypted.tmp";
        let decrypted_path = "test_decrypted.tmp";
        
        // Clean up any existing test files
        let _ = fs::remove_file(encrypted_path);
        let _ = fs::remove_file(decrypted_path);
        
        // Test encryption
        assert!(cipher.encrypt_file(input_path, encrypted_path).is_ok());
        
        // Test decryption
        assert!(cipher.decrypt_file(encrypted_path, decrypted_path).is_ok());
        
        // Verify decrypted content matches original
        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        // Clean up
        fs::remove_file(encrypted_path).ok();
        fs::remove_file(decrypted_path).ok();
    }
}