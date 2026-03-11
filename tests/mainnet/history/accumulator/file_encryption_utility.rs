
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?.as_bytes();
        
        let key_bytes = &hash_bytes[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        
        Ok(Self { key: *key })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::generate(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_random_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    salt
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let password = "secure_password_123";
    let salt = generate_random_salt();
    
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    let test_data = b"Confidential data that needs protection";
    let input_path = Path::new("test_input.txt");
    let encrypted_path = Path::new("test_encrypted.bin");
    let decrypted_path = Path::new("test_decrypted.txt");
    
    fs::write(input_path, test_data)?;
    
    encryptor.encrypt_file(input_path, encrypted_path)?;
    println!("File encrypted successfully");
    
    encryptor.decrypt_file(encrypted_path, decrypted_path)?;
    println!("File decrypted successfully");
    
    let decrypted_data = fs::read(decrypted_path)?;
    assert_eq!(test_data.as_slice(), &decrypted_data);
    
    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    process_encryption()
}