
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let input_data = fs::read(input_path)?;

    let encrypted_data: Vec<u8> = input_data
        .into_iter()
        .map(|byte| byte ^ encryption_key)
        .collect();

    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
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

    println!("Enter encryption key (leave empty for default 0xAA):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    let key_input = key_input.trim();

    let key = if key_input.is_empty() {
        None
    } else {
        match u8::from_str_radix(key_input, 16) {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid hex key, using default.");
                None
            }
        }
    };

    println!("Encrypt (e) or Decrypt (d)?");
    let mut mode = String::new();
    io::stdin().read_line(&mut mode)?;

    match mode.trim() {
        "e" => xor_encrypt_file(input_path, output_path, key),
        "d" => xor_decrypt_file(input_path, output_path, key),
        _ => {
            eprintln!("Invalid mode selected.");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption() {
        let input_data = b"Hello, World!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0xCC),
        )
        .unwrap();

        let encrypted_data = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted_data, input_data);

        let decrypted_file = NamedTempFile::new().unwrap();
        xor_decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0xCC),
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, input_data);
    }
}