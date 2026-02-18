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
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Test data for encryption and decryption";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
            
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

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

pub fn process_directory(dir_path: &str, operation: fn(&str, &str, Option<u8>) -> io::Result<()>, key: Option<u8>) -> io::Result<()> {
    let path = Path::new(dir_path);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                let input_str = file_path.to_str().unwrap();
                let output_str = format!("{}.processed", input_str);
                operation(input_str, &output_str, key)?;
                println!("Processed: {}", input_str);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, XOR encryption!";
        let key = Some(0xAA);

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap(), 
                    key).unwrap();
        
        decrypt_file(encrypted_file.path().to_str().unwrap(), 
                    decrypted_file.path().to_str().unwrap(), 
                    key).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_default_key() {
        let test_data = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    output_file.path().to_str().unwrap(), 
                    None).unwrap();
        
        let encrypted_data = fs::read(output_file.path()).unwrap();
        assert_ne!(test_data.to_vec(), encrypted_data);
    }
}use aes_gcm::{
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
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Insufficient hash length".to_string());
    }
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash_bytes[..32]);
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<EncryptionResult, String> {
    let plaintext = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_data = salt.as_bytes().to_vec();
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, &output_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<Vec<u8>, String> {
    let encrypted_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt = SaltString::from_b64(
        std::str::from_utf8(&encrypted_data[..22])
            .map_err(|e| format!("Invalid salt encoding: {}", e))?
    ).map_err(|e| format!("Invalid salt: {}", e))?;
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_start = 22;
    let nonce_end = nonce_start + NONCE_SIZE;
    let nonce = Nonce::from_slice(&encrypted_data[nonce_start..nonce_end]);
    
    let ciphertext = &encrypted_data[nonce_end..];
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), String> {
    print!("Enter input file path: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Read failed: {}", e))?;
    let input_path = input_path.trim();
    
    print!("Enter output file path: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Read failed: {}", e))?;
    let output_path = output_path.trim();
    
    print!("Enter encryption password: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Read failed: {}", e))?;
    let password = password.trim();
    
    match encrypt_file(input_path, output_path, password) {
        Ok(result) => {
            println!("Encryption successful!");
            println!("Generated nonce: {:?}", hex::encode(result.nonce));
            Ok(())
        }
        Err(e) => Err(e)
    }
}

pub fn interactive_decrypt() -> Result<(), String> {
    print!("Enter encrypted file path: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Read failed: {}", e))?;
    let input_path = input_path.trim();
    
    print!("Enter output file path: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Read failed: {}", e))?;
    let output_path = output_path.trim();
    
    print!("Enter decryption password: ");
    io::stdout().flush().map_err(|e| format!("Flush failed: {}", e))?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Read failed: {}", e))?;
    let password = password.trim();
    
    match decrypt_file(input_path, output_path, password) {
        Ok(plaintext) => {
            println!("Decryption successful!");
            println!("Decrypted {} bytes", plaintext.len());
            Ok(())
        }
        Err(e) => Err(e)
    }
}