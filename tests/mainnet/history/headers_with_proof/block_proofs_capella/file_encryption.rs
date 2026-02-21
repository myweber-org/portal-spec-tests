
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
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
        self.key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[self.key_index];
                self.key_index = (self.key_index + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let original_text = b"Hello, World! This is a test message.";
        let key = "secret_key";

        let mut cipher = XorCipher::new(key);
        
        let mut encrypted = original_text.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= key.as_bytes()[i % key.len()];
        }

        let mut decrypted = encrypted.clone();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= key.as_bytes()[i % key.len()];
        }

        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_content = b"Test file content for encryption demonstration";
        let key = "test_password_123";
        
        let source_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(source_file.path(), test_content)?;

        let mut cipher = XorCipher::new(key);
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        
        let mut cipher2 = XorCipher::new(key);
        cipher2.decrypt_file(encrypted_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_content.to_vec(), decrypted_content);

        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, Rust!";
        let key = Some(0xAA);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data {
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
    
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' not found", input_file);
        std::process::exit(1);
    }

    let key = match std::env::var("ENCRYPTION_KEY") {
        Ok(val) => val.parse().unwrap_or(DEFAULT_KEY),
        Err(_) => DEFAULT_KEY,
    };

    process_file(input_file, output_file, key)?;

    println!("{} completed successfully for '{}'", 
             if mode == "encrypt" { "Encryption" } else { "Decryption" },
             input_file);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let content = b"Hello, World!";
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content)?;

        let input_path = temp_file.path().to_str().unwrap();
        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path().to_str().unwrap();

        process_file(input_path, output_path, DEFAULT_KEY)?;

        let mut encrypted = Vec::new();
        fs::File::open(output_path)?.read_to_end(&mut encrypted)?;
        assert_ne!(encrypted, content);

        let mut temp_file2 = NamedTempFile::new()?;
        temp_file2.write_all(&encrypted)?;
        let input_path2 = temp_file2.path().to_str().unwrap();
        let output_file2 = NamedTempFile::new()?;
        let output_path2 = output_file2.path().to_str().unwrap();

        process_file(input_path2, output_path2, DEFAULT_KEY)?;

        let mut decrypted = Vec::new();
        fs::File::open(output_path2)?.read_to_end(&mut decrypted)?;
        assert_eq!(decrypted, content);

        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
        }
    }

    pub fn encrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    pub fn decrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_index];
            result.push(byte ^ key_byte);
            self.key_index = (self.key_index + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_index = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.encrypt_bytes(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let test_content = b"Confidential data that needs protection.";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(test_content)?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        encrypt_file(input_file.path(), encrypted_file.path(), key)?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), key)?;
        
        let mut decrypted_content = Vec::new();
        let mut decrypted_file_handle = fs::File::open(decrypted_file.path())?;
        decrypted_file_handle.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(test_content.to_vec(), decrypted_content);
        Ok(())
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::{decode, encode};
use rand::RngCore;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct EncryptionResult {
    pub ciphertext: String,
    pub iv: String,
}

pub fn encrypt_data(plaintext: &str, key: &[u8; 32]) -> Result<EncryptionResult, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    Ok(EncryptionResult {
        ciphertext: encode(&ciphertext),
        iv: encode(iv),
    })
}

pub fn decrypt_data(ciphertext_hex: &str, iv_hex: &str, key: &[u8; 32]) -> Result<String, Box<dyn Error>> {
    let ciphertext = decode(ciphertext_hex)?;
    let iv = decode(iv_hex)?;

    if iv.len() != 16 {
        return Err("IV must be 16 bytes".into());
    }

    let decrypted = Aes256CbcDec::new(key.into(), &iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = "Sensitive data that needs protection";

        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &encrypted.iv, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42; 16];
        let plaintext = "test";

        let result = encrypt_data(plaintext, &short_key);
        assert!(result.is_err());
    }
}