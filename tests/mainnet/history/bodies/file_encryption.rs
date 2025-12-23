
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> [key]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];
    let key = if args.len() > 4 {
        args[4].parse::<u8>().ok()
    } else {
        None
    };
    
    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_file);
        std::process::exit(1);
    }
    
    let result = match operation.as_str() {
        "encrypt" => encrypt_file(input_file, output_file, key),
        "decrypt" => decrypt_file(input_file, output_file, key),
        _ => {
            eprintln!("Error: Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    };
    
    match result {
        Ok(_) => println!("Operation completed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}