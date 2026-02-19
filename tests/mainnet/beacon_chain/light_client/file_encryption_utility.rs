use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("strong_encryption_key_123!");
        let test_data = b"Hello, this is a secret message!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XORCipher::new("test_key_890");
        let original_content = b"Sample file content for encryption test";
        
        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_123").is_ok());
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
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
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

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    
    output_file
        .write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    
    output_file
        .write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if buffer.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt = buffer[..SALT_SIZE].try_into().unwrap();
    let nonce = buffer[SALT_SIZE..SALT_SIZE + NONCE_SIZE]
        .try_into()
        .unwrap();
    let ciphertext = buffer[SALT_SIZE + NONCE_SIZE..].to_vec();
    
    let encrypted = EncryptionResult {
        ciphertext,
        nonce,
        salt,
    };
    
    let decrypted_data = decrypt_data(&encrypted, password)?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file
        .write_all(&decrypted_data)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(())
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let password = password.trim();
    
    encrypt_file(input_path, output_path, password)?;
    println!("File encrypted successfully!");
    
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter encrypted file path:");
    let mut input_path = String::new();
    io::stdin()
        .read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin()
        .read_line(&mut output_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter password:");
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let password = password.trim();
    
    decrypt_file(input_path, output_path, password)?;
    println!("File decrypted successfully!");
    
    Ok(())
}