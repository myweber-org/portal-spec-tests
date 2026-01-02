
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

    fn next_key_byte(&mut self) -> u8 {
        let byte = self.key[self.key_position];
        self.key_position = (self.key_position + 1) % self.key.len();
        byte
    }

    pub fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| byte ^ self.next_key_byte())
            .collect()
    }

    pub fn process_file(&mut self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data = self.process_bytes(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> Result<(), String> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err(format!("Input file does not exist: {}", input_path));
    }

    let mut cipher = XorCipher::new(key);
    cipher
        .process_file(Path::new(input_path), Path::new(output_path))
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> Result<(), String> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original_text = b"Hello, World! This is a test message.";
        let key = "secret_key";

        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.process_bytes(original_text);

        cipher.reset();
        let decrypted = cipher.process_bytes(&encrypted);

        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let original_content = b"Sample file content for encryption test";
        let key = "test_password";

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let encrypted_content = fs::read(output_file.path()).unwrap();
        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new().unwrap();
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }

    #[test]
    fn test_nonexistent_file() {
        let result = encrypt_file("nonexistent.txt", "output.txt", "key");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let cipher_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ cipher_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer[..bytes_read].iter_mut() {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original = b"Hello, World!";
        let key = 0x42;
        
        let encrypted: Vec<u8> = original.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        let decrypted: Vec<u8> = encrypted.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_file_operations() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), b"Test data")?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            restored_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        let restored_data = fs::read(restored_file.path())?;
        assert_eq!(b"Test data", restored_data.as_slice());
        
        Ok(())
    }
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err(format!("Input file '{}' does not exist", input_path));
    }

    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;

    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let original_content = b"Hello, this is a secret message!";
        let key = b"mysecretkey";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
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
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, DEFAULT_KEY)?;
    
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::ToHex;
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
        ciphertext: ciphertext.encode_hex(),
        iv: iv.encode_hex(),
    })
}

pub fn decrypt_data(ciphertext_hex: &str, iv_hex: &str, key: &[u8; 32]) -> Result<String, Box<dyn Error>> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let iv = hex::decode(iv_hex)?;

    if iv.len() != 16 {
        return Err("IV must be 16 bytes".into());
    }

    let decrypted = Aes256CbcDec::new(key.into(), iv.as_slice().into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = "Sensitive data requiring protection";

        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &encrypted.iv, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42u8; 16];
        let result = encrypt_data("test", &short_key);
        assert!(result.is_err());
    }
}