
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }

    process_file(input_path, output_path, DEFAULT_KEY)?;
    
    println!("File processed successfully with key 0x{:02X}", DEFAULT_KEY);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0x55;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let input_content = b"Hello, World!";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), input_content)?;
        
        process_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;
        
        let processed_content = fs::read(output_file.path())?;
        assert_ne!(processed_content, input_content);
        
        let mut double_processed = processed_content.clone();
        xor_cipher(&mut double_processed, DEFAULT_KEY);
        assert_eq!(double_processed, input_content);
        
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let ciphertext = self.cipher.encrypt(nonce, plaintext)?;
        Ok(ciphertext)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn process_file_encryption() -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let original_data = b"Sensitive file content requiring protection";
    
    let encrypted = encryptor.encrypt_data(original_data)?;
    println!("Encrypted data length: {} bytes", encrypted.len());
    
    let decrypted = encryptor.decrypt_data(&encrypted)?;
    assert_eq!(original_data.to_vec(), decrypted);
    println!("Decryption successful, data integrity verified");
    
    Ok(())
}