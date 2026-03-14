
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
    fn test_xor_encryption_roundtrip() {
        let test_data = b"Hello, XOR encryption!";
        let key = b"secret_key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
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
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use base64::{Engine as _, engine::general_purpose};

const KEY: &[u8] = b"secret-encryption-key-2024";

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let encrypted = xor_cipher(&buffer, KEY);
    let encoded = general_purpose::STANDARD.encode(encrypted);
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(encoded.as_bytes())?;
    
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut encoded = String::new();
    file.read_to_string(&mut encoded)?;
    
    let decoded = general_purpose::STANDARD.decode(encoded.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let decrypted = xor_cipher(&decoded, KEY);
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&decrypted)?;
    
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];
    
    match operation.as_str() {
        "encrypt" => {
            if let Err(e) = encrypt_file(input_file, output_file) {
                eprintln!("Encryption failed: {}", e);
                std::process::exit(1);
            }
            println!("File encrypted successfully: {}", output_file);
        }
        "decrypt" => {
            if let Err(e) = decrypt_file(input_file, output_file) {
                eprintln!("Decryption failed: {}", e);
                std::process::exit(1);
            }
            println!("File decrypted successfully: {}", output_file);
        }
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}