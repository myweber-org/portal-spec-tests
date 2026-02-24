
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Result<Self, String> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Ok(FileEncryptor { cipher })
    }

    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        let ciphertext = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

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
        let key = FileEncryptor::generate_key();
        let encryptor = FileEncryptor::new(&key).unwrap();

        let original_content = b"Test data for encryption and decryption";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
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

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let params = Params::new(15 * 1024, 2, 1, None)
        .map_err(|e| format!("Failed to create Argon2 params: {}", e))?;
    
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Password hashing failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    
    if hash_bytes.len() < 32 {
        return Err("Generated hash too short".to_string());
    }
    
    let key_bytes: [u8; 32] = hash_bytes[..32]
        .try_into()
        .map_err(|_| "Failed to extract 32-byte key".to_string())?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted_data = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let result = EncryptionResult {
        encrypted_data: encrypted_data.clone(),
        nonce: nonce_bytes,
        salt,
    };
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&salt);
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, &output_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let encrypted_data = fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain salt and nonce".to_string());
    }
    
    let salt = &encrypted_data[..SALT_SIZE];
    let nonce_bytes = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &decrypted_data)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(decrypted_data)
}

pub fn encrypt_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read input stream: {}", e))?;
    
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted_data = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    writer.write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    writer.write_all(&nonce_bytes)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    writer.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        nonce: nonce_bytes,
        salt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let test_data = b"Test data for encryption and decryption";
        let password = "secure_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypt_result = encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password,
        ).unwrap();
        
        assert_eq!(encrypt_result.salt.len(), SALT_SIZE);
        assert_eq!(encrypt_result.nonce.len(), NONCE_SIZE);
        
        let decrypted_data = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
        ).unwrap();
        
        assert_eq!(decrypted_data, test_data);
        
        let wrong_password_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            "wrong_password",
        );
        
        assert!(wrong_password_result.is_err());
    }
    
    #[test]
    fn test_stream_encryption() {
        let test_data = b"Stream encryption test data";
        let password = "stream_password";
        
        let mut input = std::io::Cursor::new(test_data.to_vec());
        let mut output = Vec::new();
        
        let encrypt_result = encrypt_stream(&mut input, &mut output, password).unwrap();
        
        assert_eq!(encrypt_result.salt.len(), SALT_SIZE);
        assert_eq!(encrypt_result.nonce.len(), NONCE_SIZE);
        assert!(!encrypt_result.encrypted_data.is_empty());
        assert_eq!(output.len(), SALT_SIZE + NONCE_SIZE + encrypt_result.encrypted_data.len());
    }
}