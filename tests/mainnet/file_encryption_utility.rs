
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key<Aes256Gcm>, Box<dyn std::error::Error>> {
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), salt)?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short for key")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;
    
    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(salt.as_str().as_bytes())?;
    output_file.write_all(&nonce_bytes)?;
    output_file.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;
    
    if encrypted_data.len() < 22 {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt_str = std::str::from_utf8(&encrypted_data[..22])?;
    let salt = SaltString::new(salt_str)?;
    let nonce_start = 22;
    let nonce_end = nonce_start + NONCE_SIZE;
    let ciphertext_start = nonce_end;
    
    let nonce_bytes: [u8; NONCE_SIZE] = encrypted_data[nonce_start..nonce_end]
        .try_into()
        .map_err(|_| "Invalid nonce size")?;
    let ciphertext = &encrypted_data[ciphertext_start..];
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(())
}

pub fn encrypt_data(
    data: &[u8],
    password: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let mut combined_data = Vec::new();
    combined_data.extend_from_slice(salt.as_str().as_bytes());
    combined_data.extend_from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    combined_data.extend_from_slice(&ciphertext);
    
    Ok(EncryptionResult {
        ciphertext: combined_data,
        nonce: nonce_bytes,
    })
}

pub fn interactive_encrypt() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    encrypt_file(input_path, output_path, password)?;
    println!("File encrypted successfully!");
    
    Ok(())
}