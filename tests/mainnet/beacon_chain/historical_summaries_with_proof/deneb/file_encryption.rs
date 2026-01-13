
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let key_slice = &encrypted_data[encrypted_data.len() - 32..];
    let key = Key::<Aes256Gcm>::from_slice(key_slice);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let ciphertext = &encrypted_data[..encrypted_data.len() - 32];
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct FileEncryptor {
    key: [u8; 32],
    iv: [u8; 16],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { key, iv }
    }

    pub fn with_key_iv(key: [u8; 32], iv: [u8; 16]) -> Self {
        Self { key, iv }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let ciphertext = Aes256CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let plaintext = Aes256CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    pub fn get_iv(&self) -> &[u8; 16] {
        &self.iv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, this is a secret message!";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.txt";

        // Write test data
        fs::write(input_path, test_data).unwrap();

        let encryptor = FileEncryptor::new();
        
        // Encrypt
        encryptor.encrypt_file(input_path, encrypted_path).unwrap();
        
        // Decrypt
        encryptor.decrypt_file(encrypted_path, decrypted_path).unwrap();
        
        // Verify
        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);

        // Cleanup
        fs::remove_file(input_path).unwrap();
        fs::remove_file(encrypted_path).unwrap();
        fs::remove_file(decrypted_path).unwrap();
    }
}