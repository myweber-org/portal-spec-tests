
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
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
        let original_text = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
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
        assert_eq!(decrypted_content, test_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> [key]", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];
    let key = args.get(4).and_then(|k| k.parse::<u8>().ok());

    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' not found", input_file);
        std::process::exit(1);
    }

    match operation.as_str() {
        "encrypt" => encrypt_file(input_file, output_file, key)?,
        "decrypt" => decrypt_file(input_file, output_file, key)?,
        _ => {
            eprintln!("Error: Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }

    println!("Operation completed successfully: {} -> {}", input_file, output_file);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        xor_cipher(&mut data, DEFAULT_KEY);
        xor_cipher(&mut data, DEFAULT_KEY);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        
        let test_data = b"Hello, World!";
        temp_input.write_all(test_data)?;
        
        let input_path = temp_input.path().to_str().unwrap();
        let output_path = temp_output.path().to_str().unwrap();
        
        encrypt_file(input_path, output_path, Some(0xCC))?;
        let encrypted = fs::read(output_path)?;
        assert_ne!(encrypted, test_data);
        
        decrypt_file(output_path, input_path, Some(0xCC))?;
        let decrypted = fs::read(input_path)?;
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Encrypts or decrypts a file using XOR with the provided key.
///
/// # Arguments
///
/// * `input_path` - Path to the input file.
/// * `output_path` - Path where the output (encrypted/decrypted) file will be saved.
/// * `key` - The byte used as the XOR key.
///
/// # Returns
///
/// * `io::Result<()>` - Ok if successful, Err otherwise.
pub fn xor_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    // Apply XOR operation to each byte
    for byte in &mut buffer {
        *byte ^= key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key_byte>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let key_byte = args[3]
        .parse::<u8>()
        .expect("Key must be a number between 0 and 255");

    xor_file(input_path, output_path, key_byte)?;
    println!("Operation completed successfully.");
    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_data = key.to_vec();
    output_data.extend_from_slice(nonce);
    output_data.extend_from_slice(&ciphertext);

    fs::write(output_path, output_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    if data.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }

    let key = &data[0..32];
    let nonce = &data[32..44];
    let ciphertext = &data[44..];

    let cipher = Aes256Gcm::new_from_slice(key)?;
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let mut plain_file = NamedTempFile::new().unwrap();
        writeln!(plain_file, "Sensitive data: {}", "TestContent123").unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encrypt_file(plain_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();

        let original = fs::read_to_string(plain_file.path()).unwrap();
        let decrypted = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(original, decrypted);
    }
}