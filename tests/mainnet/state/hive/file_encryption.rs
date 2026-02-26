
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::generate(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption successful. Key: {}", hex::encode(key));
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key_hex: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(output_path, plaintext)?;
    println!("Decryption successful.");
    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    println!("Encrypt (e) or Decrypt (d)?");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    println!("Enter encryption key (leave empty for default):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    
    let key = if key_input.trim().is_empty() {
        None
    } else {
        match key_input.trim().parse::<u8>() {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid key, using default");
                None
            }
        }
    };
    
    match choice.trim() {
        "e" | "E" => encrypt_file(input_path, output_path, key),
        "d" | "D" => decrypt_file(input_path, output_path, key),
        _ => {
            eprintln!("Invalid choice");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World! This is a test.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(test_data.to_vec(), encrypted);
        
        let decrypted: Vec<u8> = encrypted.iter().map(|b| b ^ DEFAULT_KEY).collect();
        assert_eq!(test_data.to_vec(), decrypted);
    }
}