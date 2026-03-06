
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
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| format!("Password hashing failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.unwrap().as_bytes();
    let key_slice = &hash_bytes[..32];
    
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let encrypted_data = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce,
    })
}

pub fn decrypt_data(
    encrypted_data: &[u8],
    password: &str,
    salt: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    cipher
        .decrypt(Nonce::from_slice(nonce), encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let result = encrypt_data(&buffer, password)?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file.write_all(&result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file.write_all(&result.encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if file_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("File too short to contain salt and nonce".to_string());
    }
    
    let salt = &file_data[..SALT_LENGTH];
    let nonce = &file_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let encrypted_data = &file_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let decrypted_data = decrypt_data(encrypted_data, password, salt, nonce)?;
    
    fs::write(output_path, decrypted_data)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let decrypted = decrypt_data(
            &encrypted.encrypted_data,
            password,
            &encrypted.salt,
            &encrypted.nonce,
        ).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let original_content = b"File content to encrypt";
        let password = "file_password_456";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let nonce = Nonce::from_slice(&generate_nonce());
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(nonce.as_slice())
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte;
    }
    
    Key::<Aes256Gcm>::from_slice(&key_bytes).clone()
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        let password = "strong_password_123";
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original_content = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            "correct_password"
        ).unwrap();
        
        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            "wrong_password"
        );
        
        assert!(result.is_err());
    }
}
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
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher.encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext = &encrypted_data[NONCE_SIZE..];
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let decrypted_data = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)?.write_all(&decrypted_data)?;
        
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        generate_random_bytes(SALT_SIZE).try_into().unwrap()
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = FileEncryptor::generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file_content = Vec::new();
    File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?
        .read_to_end(&mut file_content)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, file_content.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let mut ciphertext = Vec::new();
    File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?
        .read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce_obj = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce_obj, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(plaintext)
}

pub fn generate_random_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    password
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data for encryption test";
        let password = "TestPassword123!";

        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.salt,
            &enc_result.nonce,
        ).unwrap();

        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_key_derivation() {
        let password = "MySecurePassword";
        let salt = [0u8; SALT_LENGTH];
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key1, key2);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
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
                "Encrypted file too short"
            ));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt)
            .expect("Failed to create encryptor");
        
        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        let decrypted_file = NamedTempFile::new().unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(decrypted_content, original_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac_array, Pbkdf2};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8; SALT_LENGTH]) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    Key::<Aes256Gcm>::from_slice(&key).clone()
}

pub fn encrypt_data(plaintext: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    match cipher.encrypt(nonce, plaintext) {
        Ok(ciphertext) => Ok(EncryptionResult {
            ciphertext,
            salt,
            nonce: nonce_bytes,
        }),
        Err(e) => Err(format!("Encryption failed: {}", e)),
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    password: &str,
    salt: &[u8; SALT_LENGTH],
    nonce: &[u8; NONCE_LENGTH],
) -> Result<Vec<u8>, String> {
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    match cipher.decrypt(nonce, ciphertext) {
        Ok(plaintext) => Ok(plaintext),
        Err(e) => Err(format!("Decryption failed: {}", e)),
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let result = encrypt_data(&buffer, password)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    if buffer.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt: [u8; SALT_LENGTH] = buffer[..SALT_LENGTH].try_into().unwrap();
    let nonce: [u8; NONCE_LENGTH] = buffer[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH]
        .try_into()
        .unwrap();
    let ciphertext = &buffer[SALT_LENGTH + NONCE_LENGTH..];
    
    let plaintext = decrypt_data(ciphertext, password, &salt, &nonce)?;
    
    fs::write(output_path, plaintext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}