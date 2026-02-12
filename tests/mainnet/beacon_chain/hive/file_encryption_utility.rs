use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

#[derive(Debug)]
pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::generate(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let nonce = Nonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| format!("Decryption failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let key = [0x42; 32];
        let plaintext = b"Test encryption data";
        let result = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::Aes256Gcm).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let key = [0x24; 32];
        let plaintext = b"Another test message";
        let result = encrypt_data(plaintext, &key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce, CipherType::ChaCha20Poly1305).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher.encrypt(&nonce, data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = nonce.to_vec();
        output.extend(encrypted_data);
        
        fs::write(output_path, output)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new("test_password123").unwrap();
        let test_data = b"Hello, secure world!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            output_file.write_all(&processed_buffer)?;
        }

        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let cipher = XORCipher::new(key);

        let test_data = b"Hello, this is a test message for XOR encryption!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_idx = 0;
        cipher.xor_transform(&mut encrypted, &mut key_idx);

        let mut decrypted = encrypted.clone();
        key_idx = 0;
        cipher.xor_transform(&mut decrypted, &mut key_idx);

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let cipher = XORCipher::new(key);

        let original_content = b"Sample file content for encryption testing.";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original_content)?;

        cipher.encrypt_file(input_file.path(), output_file.path())?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }
}
use aes_gcm::{
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
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let salt_string = SaltString::encode_b64(salt)?;
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)?;
        let key_material = password_hash.hash.ok_or("Failed to derive key")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_material.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_SIZE] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    salt
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter password for encryption:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let salt = generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = Path::new(output_path.trim());
    
    encryptor.encrypt_file(input_path, output_path)?;
    println!("File encrypted successfully!");
    
    println!("Salt (save this for decryption):");
    println!("{}", base64::encode(&salt));
    
    Ok(())
}

pub fn process_decryption() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter password for decryption:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    println!("Enter salt (base64 encoded):");
    let mut salt_b64 = String::new();
    io::stdin().read_line(&mut salt_b64)?;
    let salt = base64::decode(salt_b64.trim())?;
    
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    println!("Enter encrypted file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = Path::new(output_path.trim());
    
    encryptor.decrypt_file(input_path, output_path)?;
    println!("File decrypted successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "test_password_123";
        let salt = generate_salt();
        
        let original_content = b"This is a secret message that needs encryption!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_content).unwrap();
        
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        encryptor.encrypt_file(temp_input.path(), temp_encrypted.path()).unwrap();
        encryptor.decrypt_file(temp_encrypted.path(), temp_decrypted.path()).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
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

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: 32,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
    key
}

pub fn encrypt_data(plaintext: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    
    let key_material = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
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
    encrypted: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, String> {
    let key_material = derive_key(password, &encrypted.salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    match cipher.decrypt(nonce, encrypted.ciphertext.as_ref()) {
        Ok(plaintext) => Ok(plaintext),
        Err(e) => Err(format!("Decryption failed: {}", e)),
    }
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted = encrypt_data(&plaintext, password)?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file.write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file.write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt = encrypted_data[..SALT_LENGTH].try_into()
        .map_err(|_| "Failed to extract salt")?;
    let nonce = encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH].try_into()
        .map_err(|_| "Failed to extract nonce")?;
    let ciphertext = encrypted_data[SALT_LENGTH + NONCE_LENGTH..].to_vec();
    
    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let plaintext = decrypt_data(&encrypted, password)?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(plaintext, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption() {
        let test_data = b"Test file content for encryption";
        let password = "file_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), output_file.path(), password).unwrap();
        decrypt_file(output_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Sensitive information";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}