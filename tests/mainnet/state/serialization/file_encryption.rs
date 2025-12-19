
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

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
    let decryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let encrypted_data = fs::read(input_path)?;
    let decrypted_data: Vec<u8> = encrypted_data
        .iter()
        .map(|byte| byte ^ decryption_key)
        .collect();
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

pub fn encrypt_string(text: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    text.bytes()
        .map(|byte| byte ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let decryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|byte| (byte ^ decryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let original = "Hello, World!";
        let key = Some(0x42);
        
        let encrypted = encrypt_string(original, key);
        let decrypted = decrypt_string(&encrypted, key);
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let original_content = b"Secret file content";
        let key = Some(0x99);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
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
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
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
        return Err(io::Error::new(io::ErrorKind::NotFound, "Input file not found"));
    }
    
    match operation.as_str() {
        "encrypt" => encrypt_file(input_file, output_file, key),
        "decrypt" => decrypt_file(input_file, output_file, key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let mut input_file = NamedTempFile::new().unwrap();
        let mut output_file = NamedTempFile::new().unwrap();
        let mut final_file = NamedTempFile::new().unwrap();
        
        input_file.write_all(original_data).unwrap();
        
        encrypt_file(input_file.path().to_str().unwrap(),
                    output_file.path().to_str().unwrap(),
                    Some(0x42)).unwrap();
        
        decrypt_file(output_file.path().to_str().unwrap(),
                    final_file.path().to_str().unwrap(),
                    Some(0x42)).unwrap();
        
        let decrypted_data = fs::read(final_file.path()).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
}