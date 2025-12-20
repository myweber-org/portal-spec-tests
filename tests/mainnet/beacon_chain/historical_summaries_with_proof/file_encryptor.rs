
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let encrypted_data = self.cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, encrypted_data)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let decrypted_data = self.cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)
    }
}

pub fn generate_key_file(path: &Path) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    let key: [u8; 32] = rand::random();
    file.write_all(&key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Test encryption data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    password: String,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
        }
    }

    fn derive_key(&self, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
        let params = Params::new(65536, 2, 1, Some(32))
            .map_err(|e| format!("Failed to create Argon2 params: {}", e))?;
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params,
        );

        let salt_str = SaltString::encode_b64(salt)
            .map_err(|e| format!("Failed to encode salt: {}", e))?;
        
        let password_hash = argon2
            .hash_password(self.password.as_bytes(), &salt_str)
            .map_err(|e| format!("Password hashing failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        if hash_bytes.len() < 32 {
            return Err("Generated hash too short".to_string());
        }
        
        let key_slice = &hash_bytes[..32];
        Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut rng = OsRng;
        let salt: [u8; SALT_SIZE] = rng.random();
        let nonce: [u8; NONCE_SIZE] = rng.random();

        let key = self.derive_key(&salt)?;
        let cipher = Aes256Gcm::new(&key);
        
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output.write_all(&salt)
            .map_err(|e| format!("Failed to write salt: {}", e))?;
        output.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
            return Err("Encrypted file too short".to_string());
        }

        let salt = &encrypted_data[..SALT_SIZE];
        let nonce = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
        let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];

        let key = self.derive_key(salt)?;
        let cipher = Aes256Gcm::new(&key);
        
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }
}

pub fn process_encryption() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> <password>", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);
    let password = &args[4];

    let encryptor = FileEncryptor::new(password);

    match operation.as_str() {
        "encrypt" => {
            encryptor.encrypt_file(input_path, output_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            println!("File encrypted successfully");
        }
        "decrypt" => {
            encryptor.decrypt_file(input_path, output_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            println!("File decrypted successfully");
        }
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "test_password_123!";
        let plaintext = b"This is a secret message that needs encryption";
        
        let encryptor = FileEncryptor::new(password);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}