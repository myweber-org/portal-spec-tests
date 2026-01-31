use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let entries = fs::read_dir(dir_path)?;
    let operation = if encrypt { "encrypted" } else { "decrypted" };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let input_str = path.to_str().unwrap();
            let output_name = format!("{}_{}", operation, path.file_name().unwrap().to_str().unwrap());
            let output_path = path.with_file_name(output_name);

            if encrypt {
                encrypt_file(input_str, output_path.to_str().unwrap(), key)?;
            } else {
                decrypt_file(input_str, output_path.to_str().unwrap(), key)?;
            }
            
            println!("Processed: {}", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Test data for encryption";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_data).unwrap();

        let encrypted_path = temp_file.path().with_extension("enc");
        let decrypted_path = temp_file.path().with_extension("dec");

        encrypt_file(temp_file.path().to_str().unwrap(), 
                    encrypted_path.to_str().unwrap(), 
                    Some(0xCC)).unwrap();
        
        decrypt_file(encrypted_path.to_str().unwrap(), 
                    decrypted_path.to_str().unwrap(), 
                    Some(0xCC)).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
}