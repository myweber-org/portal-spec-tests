use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, ParamsBuilder
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        
        let params = ParamsBuilder::new()
            .m_cost(19456)
            .t_cost(2)
            .p_cost(1)
            .output_len(32)
            .build()?;
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params,
        );
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_LENGTH] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn process_directory(
    encryptor: &FileEncryptor,
    dir_path: &Path,
    operation: Operation,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut processed_count = 0;
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            match operation {
                Operation::Encrypt => {
                    if !extension.ends_with(".enc") {
                        let output_path = path.with_extension(format!("{}.enc", extension));
                        encryptor.encrypt_file(&path, &output_path)?;
                        processed_count += 1;
                    }
                }
                Operation::Decrypt => {
                    if extension.ends_with(".enc") {
                        let original_extension = extension.trim_end_matches(".enc");
                        let output_path = path.with_extension(original_extension);
                        encryptor.decrypt_file(&path, &output_path)?;
                        processed_count += 1;
                    }
                }
            }
        }
    }
    
    Ok(processed_count)
}

pub enum Operation {
    Encrypt,
    Decrypt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_encryption_decryption() {
        let test_password = "secure_password_123";
        let encryptor = FileEncryptor::new(test_password).unwrap();
        
        let temp_dir = tempdir().unwrap();
        let original_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.txt.enc");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");
        
        let original_content = b"Secret data that needs protection";
        fs::write(&original_path, original_content).unwrap();
        
        encryptor.encrypt_file(&original_path, &encrypted_path).unwrap();
        encryptor.decrypt_file(&encrypted_path, &decrypted_path).unwrap();
        
        let decrypted_content = fs::read(&decrypted_path).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}