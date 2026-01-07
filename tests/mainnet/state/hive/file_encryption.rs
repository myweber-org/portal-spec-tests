
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < 32 + NONCE_SIZE {
        return Err("File too short to contain valid encrypted data".into());
    }

    let (ciphertext, key_slice) = data.split_at(data.len() - 32);
    let key = Key::<Aes256Gcm>::from_slice(key_slice);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let original_content = b"Test data for encryption roundtrip";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}