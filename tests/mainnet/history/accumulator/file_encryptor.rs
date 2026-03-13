
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file() -> io::Result<()> {
    let key = b"secret_key";
    let input_file = "plaintext.txt";
    let encrypted_file = "encrypted.bin";
    let decrypted_file = "decrypted.txt";

    if !Path::new(input_file).exists() {
        let sample_text = b"This is a sample text to demonstrate XOR encryption.";
        fs::write(input_file, sample_text)?;
        println!("Created sample input file: {}", input_file);
    }

    xor_encrypt_file(input_file, encrypted_file, key)?;
    println!("File encrypted successfully: {}", encrypted_file);

    xor_decrypt_file(encrypted_file, decrypted_file, key)?;
    println!("File decrypted successfully: {}", decrypted_file);

    let original = fs::read(input_file)?;
    let restored = fs::read(decrypted_file)?;
    
    if original == restored {
        println!("Verification: Encryption/decryption cycle successful!");
    } else {
        println!("Warning: Data mismatch after decryption!");
    }

    Ok(())
}

fn main() {
    if let Err(e) = process_file() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_data = key.to_vec();
    output_data.extend_from_slice(&ciphertext);
    fs::write(output_path, output_data)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    if data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }

    let (key_bytes, ciphertext) = data.split_at(32);
    let key = key_bytes.try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}use std::fs;
use std::io::{self, Read, Write};

const KEY: &[u8] = b"secret-encryption-key-2024";

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn encode_base64(data: &[u8]) -> String {
    base64::encode(data)
}

fn decode_base64(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode(encoded)
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let encrypted = xor_cipher(&buffer, KEY);
    let encoded = encode_base64(&encrypted);
    
    fs::write(output_path, encoded)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let encoded = fs::read_to_string(input_path)?;
    let encrypted = decode_base64(&encoded).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e.to_string())
    })?;
    
    let decrypted = xor_cipher(&encrypted, KEY);
    fs::write(output_path, decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Hello, this is a secret message!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), test_data).unwrap();
        
        encrypt_file(temp_input.path().to_str().unwrap(), 
                    temp_encrypted.path().to_str().unwrap()).unwrap();
        decrypt_file(temp_encrypted.path().to_str().unwrap(),
                    temp_decrypted.path().to_str().unwrap()).unwrap();
        
        let result = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; 4096];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer.iter_mut().take(bytes_read) {
            *byte ^= encryption_key;
        }
        
        output_file.write_all(&buffer[..bytes_read])?;
    }
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World! This is a test message.";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_data).unwrap();
        
        encrypt_file(test_file, encrypted_file, Some(0x42)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0x42)).unwrap();
        
        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
        
        fs::remove_file(test_file).ok();
        fs::remove_file(encrypted_file).ok();
        fs::remove_file(decrypted_file).ok();
    }
}