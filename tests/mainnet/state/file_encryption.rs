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
    let original = "test_data.txt";
    let encrypted = "encrypted.bin";
    let decrypted = "decrypted.txt";

    if !Path::new(original).exists() {
        let mut file = fs::File::create(original)?;
        file.write_all(b"Sample data for encryption test\nSecond line of content")?;
    }

    xor_encrypt_file(original, encrypted, key)?;
    xor_decrypt_file(encrypted, decrypted, key)?;

    let original_content = fs::read_to_string(original)?;
    let decrypted_content = fs::read_to_string(decrypted)?;

    assert_eq!(original_content, decrypted_content);
    println!("Encryption and decryption completed successfully");
    
    fs::remove_file(encrypted)?;
    fs::remove_file(decrypted)?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_file() {
        eprintln!("Error: {}", e);
    }
}