
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

pub fn generate_key_from_password(password: &str, length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(length);
    let password_bytes = password.as_bytes();
    
    for i in 0..length {
        let byte = password_bytes[i % password_bytes.len()]
            .wrapping_add((i * 7) as u8)
            .rotate_left(3);
        key.push(byte);
    }
    
    key
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Select operation:");
    println!("1. Encrypt file");
    println!("2. Decrypt file");
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    
    println!("Enter password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    
    let key = generate_key_from_password(password.trim(), 32);
    
    match choice.trim() {
        "1" => xor_encrypt_file(input_path.trim(), output_path.trim(), &key),
        "2" => xor_decrypt_file(input_path.trim(), output_path.trim(), &key),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid choice")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_symmetry() {
        let test_data = b"Hello, World! This is a test message.";
        let key = b"secretkey";
        
        let encrypted: Vec<u8> = test_data
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect();
        
        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;
        
        let test_content = b"Test file content for encryption";
        fs::write(input_file.path(), test_content)?;
        
        let key = generate_key_from_password("testpass", 16);
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            &key,
        )?;
        
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            restored_file.path().to_str().unwrap(),
            &key,
        )?;
        
        let restored_content = fs::read(restored_file.path())?;
        assert_eq!(test_content.to_vec(), restored_content);
        
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and nonce",
        ));
    }

    let (key_bytes, rest) = encrypted_data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}