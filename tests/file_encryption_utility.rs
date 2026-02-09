
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8; SALT_SIZE]) -> [u8; 32] {
    let argon2 = Argon2::default();
    let mut output_key_material = [0u8; 32];
    
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output_key_material)
        .expect("Key derivation failed");
    
    output_key_material
}

pub fn encrypt_file(
    file_path: &str,
    password: &str,
    output_path: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let plaintext = fs::read(file_path)?;
    
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let key_material = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let result = EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    };
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&result.salt)?;
    output_file.write_all(&result.nonce)?;
    output_file.write_all(&result.ciphertext)?;
    
    Ok(result)
}

pub fn decrypt_file(
    encrypted_path: &str,
    password: &str,
    output_path: &str
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(encrypted_path)?;
    
    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt = encrypted_data[..SALT_SIZE].try_into().unwrap();
    let nonce_bytes = encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE].try_into().unwrap();
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];
    
    let key_material = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)?;
    
    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter file path to encrypt: ");
    io::stdout().flush()?;
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path)?;
    let file_path = file_path.trim();
    
    print!("Enter password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let output_path = format!("{}.enc", file_path);
    
    match encrypt_file(file_path, password, &output_path) {
        Ok(_) => println!("File encrypted successfully: {}", output_path),
        Err(e) => eprintln!("Encryption failed: {}", e),
    }
    
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter encrypted file path: ");
    io::stdout().flush()?;
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path)?;
    let file_path = file_path.trim();
    
    print!("Enter password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let output_path = file_path.trim_end_matches(".enc");
    
    match decrypt_file(file_path, password, output_path) {
        Ok(_) => println!("File decrypted successfully: {}", output_path),
        Err(e) => eprintln!("Decryption failed: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption functionality";
        let password = "secure_password_123";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_data).unwrap();
        
        let encrypted_path = temp_file.path().to_str().unwrap().to_owned() + ".enc";
        
        let encrypt_result = encrypt_file(
            temp_file.path().to_str().unwrap(),
            password,
            &encrypted_path
        );
        assert!(encrypt_result.is_ok());
        
        let decrypted_path = temp_file.path().to_str().unwrap().to_owned() + ".dec";
        let decrypt_result = decrypt_file(&encrypted_path, password, &decrypted_path);
        assert!(decrypt_result.is_ok());
        
        let decrypted_data = fs::read(&decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);
        
        fs::remove_file(&encrypted_path).ok();
        fs::remove_file(&decrypted_path).ok();
    }
    
    #[test]
    fn test_wrong_password() {
        let test_data = b"Sensitive information";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_data).unwrap();
        
        let encrypted_path = temp_file.path().to_str().unwrap().to_owned() + ".enc";
        
        encrypt_file(
            temp_file.path().to_str().unwrap(),
            "correct_password",
            &encrypted_path
        ).unwrap();
        
        let decrypted_path = temp_file.path().to_str().unwrap().to_owned() + ".dec";
        let result = decrypt_file(&encrypted_path, "wrong_password", &decrypted_path);
        
        assert!(result.is_err());
        
        fs::remove_file(&encrypted_path).ok();
        fs::remove_file(&decrypted_path).ok();
    }
}