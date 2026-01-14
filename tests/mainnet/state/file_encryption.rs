
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let decrypted_data = xor_decrypt(&input_data, key);
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Secret message for encryption test";
    let key = b"encryption_key";
    
    let input_file = "test_input.txt";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.txt";
    
    fs::write(input_file, test_data)?;
    
    xor_encrypt_file(input_file, encrypted_file, key)?;
    xor_decrypt_file(encrypted_file, decrypted_file, key)?;
    
    let decrypted_content = fs::read(decrypted_file)?;
    assert_eq!(test_data.to_vec(), decrypted_content);
    
    fs::remove_file(input_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    println!("File encryption test completed successfully");
    Ok(())
}