
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    if !input_path.exists() {
        return Err("Input file does not exist".into());
    }

    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let key = b"secret_key";
        let original_data = b"Hello, this is a test message for XOR encryption!";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_data).unwrap();

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
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}