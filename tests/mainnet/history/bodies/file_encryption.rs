use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(b"unique nonce");
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher.encrypt(&nonce, plaintext)?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            let cipher = Aes256Gcm::new(&key);
            let nonce = Nonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
        CipherAlgorithm::ChaCha20Poly1305 => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            let cipher = ChaCha20Poly1305::new(&key);
            let nonce = ChaChaNonce::from_slice(nonce);
            let plaintext = cipher.decrypt(nonce, ciphertext)?;
            Ok(plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"secret message";
        let result = encrypt_data(plaintext, CipherAlgorithm::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            CipherAlgorithm::Aes256Gcm,
        )
        .unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"another secret";
        let result = encrypt_data(plaintext, CipherAlgorithm::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(
            &result.ciphertext,
            &result.nonce,
            CipherAlgorithm::ChaCha20Poly1305,
        )
        .unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, DEFAULT_KEY)?;
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_path = Path::new("test_input.txt");
        let encrypted_path = Path::new("test_encrypted.txt");
        let decrypted_path = Path::new("test_decrypted.txt");
        
        fs::write(input_path, test_data)?;
        
        process_file(input_path, encrypted_path, DEFAULT_KEY)?;
        process_file(encrypted_path, decrypted_path, DEFAULT_KEY)?;
        
        let decrypted_data = fs::read(decrypted_path)?;
        assert_eq!(decrypted_data, test_data);
        
        fs::remove_file(input_path)?;
        fs::remove_file(encrypted_path)?;
        fs::remove_file(decrypted_path)?;
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_file(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let mut ciphertext = cipher.encrypt(&nonce, data)?;
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.append(&mut ciphertext);
    
    Ok(result)
}

pub fn decrypt_file(encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() < 12 {
        return Err("Invalid encrypted data".into());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let original_data = b"Secret file content that needs protection";
        
        let encrypted = encrypt_file(original_data, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
        assert_ne!(original_data, encrypted.as_slice());
    }
}