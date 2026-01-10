use std::fs::{File, read, write};
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = read(input_path)?;
    xor_transform(&mut content, key);
    write(output_path, &content)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    encrypt_file(input_path, output_path, key)
}

fn xor_transform(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = b"secret_key";
        let original = b"Hello, XOR encryption!";
        
        let mut encrypted = original.to_vec();
        xor_transform(&mut encrypted, key);
        
        assert_ne!(encrypted, original);
        
        let mut decrypted = encrypted.clone();
        xor_transform(&mut decrypted, key);
        
        assert_eq!(decrypted, original);
    }
}