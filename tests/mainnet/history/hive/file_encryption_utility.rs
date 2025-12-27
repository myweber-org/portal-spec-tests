
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Params, Pbkdf2
};
use std::fs;
use std::io::{self, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let salt_string = SaltString::b64_encode(salt)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let params = Params {
            rounds: 100_000,
            output_length: 32,
        };
        
        let password_hash = Pbkdf2
            .hash_password_customized(password.as_bytes(), None, None, params, &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let key_bytes = password_hash.hash.ok_or_else(|| 
            io::Error::new(io::ErrorKind::InvalidData, "Failed to derive key")
        )?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        Ok(Self { key: *key })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        
        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(&nonce);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Encrypted file too short"
            ));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(output_path, plaintext)
    }
    
    pub fn generate_salt() -> [u8; SALT_LENGTH] {
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        salt
    }
}

pub fn process_encryption(
    password: &str,
    input_file: &str,
    output_file: &str,
    encrypt: bool
) -> io::Result<()> {
    let salt = FileEncryptor::generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    if encrypt {
        encryptor.encrypt_file(input_file, output_file)?;
        println!("Encryption completed successfully");
    } else {
        encryptor.decrypt_file(input_file, output_file)?;
        println!("Decryption completed successfully");
    }
    
    Ok(())
}