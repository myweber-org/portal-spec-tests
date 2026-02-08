use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        SaltString
    },
    Argon2, PasswordHasher, PasswordVerifier
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let mut output_key_material = [0u8; 32];
    
    argon2.hash_password_into(
        password.as_bytes(),
        salt,
        &mut output_key_material
    ).map_err(|e| format!("Key derivation failed: {}", e))?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&output_key_material).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?
        .read_to_end(&mut file_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    nonce: &[u8; NONCE_SIZE]
) -> Result<Vec<u8>, String> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?
        .read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption functionality";
        let password = "secure_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let enc_result = encrypt_file(
            input_file.path(),
            output_file.path(),
            password
        ).expect("Encryption should succeed");
        
        let decrypted = decrypt_file(
            output_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.salt,
            &enc_result.nonce
        ).expect("Decryption should succeed");
        
        assert_eq!(decrypted, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &str) -> Result<Key<Aes256Gcm>, String> {
    let salt_bytes = SaltString::from_b64(salt)
        .map_err(|e| format!("Invalid salt: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_bytes)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let salt = SaltString::generate(&mut OsRng).to_string();
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &str,
    nonce: &[u8; NONCE_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Failed to read output path: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    let password = password.trim();
    
    let result = encrypt_file(input_path, output_path, password)?;
    
    println!("Encryption successful!");
    println!("Salt (save this for decryption): {}", result.salt);
    println!("Nonce (hex): {}", hex::encode(result.nonce));
    
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter encrypted file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Failed to read output path: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    let password = password.trim();
    
    println!("Enter salt:");
    let mut salt = String::new();
    io::stdin().read_line(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    let salt = salt.trim();
    
    println!("Enter nonce (hex):");
    let mut nonce_hex = String::new();
    io::stdin().read_line(&mut nonce_hex)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    let nonce_bytes = hex::decode(nonce_hex.trim())
        .map_err(|e| format!("Invalid hex: {}", e))?;
    
    let nonce: [u8; NONCE_SIZE] = nonce_bytes.try_into()
        .map_err(|_| format!("Nonce must be {} bytes", NONCE_SIZE))?;
    
    decrypt_file(input_path, output_path, password, salt, &nonce)?;
    
    println!("Decryption successful!");
    
    Ok(())
}
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

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_data(result: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&result.nonce);

    cipher
        .decrypt(nonce, result.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let result = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&result.salt)
        .map_err(|e| e.to_string())?;
    output_file
        .write_all(&result.nonce)
        .map_err(|e| e.to_string())?;
    output_file
        .write_all(&result.ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    if buffer.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain encryption metadata".to_string());
    }

    let salt = buffer[..SALT_SIZE].try_into().unwrap();
    let nonce = buffer[SALT_SIZE..SALT_SIZE + NONCE_SIZE].try_into().unwrap();
    let ciphertext = buffer[SALT_SIZE + NONCE_SIZE..].to_vec();

    let result = EncryptionResult {
        ciphertext,
        nonce,
        salt,
    };

    let decrypted = decrypt_data(&result, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&decrypted)
        .map_err(|e| e.to_string())?;

    Ok(decrypted)
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter file path to encrypt:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = Path::new(input_path.trim());

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = Path::new(output_path.trim());

    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    match encrypt_file(input_path, output_path, password) {
        Ok(_) => println!("File encrypted successfully"),
        Err(e) => return Err(format!("Encryption failed: {}", e)),
    }

    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter file path to decrypt:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| e.to_string())?;
    let input_path = Path::new(input_path.trim());

    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| e.to_string())?;
    let output_path = Path::new(output_path.trim());

    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| e.to_string())?;
    let password = password.trim();

    match decrypt_file(input_path, output_path, password) {
        Ok(_) => println!("File decrypted successfully"),
        Err(e) => return Err(format!("Decryption failed: {}", e)),
    }

    Ok(())
}