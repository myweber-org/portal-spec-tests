
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
}