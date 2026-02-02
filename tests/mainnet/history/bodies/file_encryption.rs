use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(key.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() < 48 {
        return Err("Invalid encrypted data length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&encrypted_data[..32]);
    let nonce = Nonce::from_slice(&encrypted_data[32..44]);
    let ciphertext = &encrypted_data[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_tampered_data() {
        let original_data = b"Another secret";
        let mut encrypted = encrypt_data(original_data).unwrap();
        
        encrypted[50] ^= 0xFF;
        
        let result = decrypt_data(&encrypted);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut rng = OsRng;
    let nonce_bytes: [u8; NONCE_SIZE] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().enumerate() {
        key[i % 32] ^= byte;
    }
    
    *Key::<Aes256Gcm>::from_slice(&key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let password = "strong_password_123";
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        )
        .unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&salt).map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data).map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("File too short to contain salt and IV".to_string());
    }

    let (salt_data, rest) = encrypted_data.split_at(SALT_LENGTH);
    let (iv_data, ciphertext) = rest.split_at(IV_LENGTH);

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    salt.copy_from_slice(salt_data);
    iv.copy_from_slice(iv_data);

    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);

    let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
    let plaintext = cipher.decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext).map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, this is a secret message!";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.txt";
        let password = "strong_password_123";

        fs::write(input_path, test_data).unwrap();

        encrypt_file(input_path, encrypted_path, password).unwrap();
        decrypt_file(encrypted_path, decrypted_path, password).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);

        fs::remove_file(input_path).ok();
        fs::remove_file(encrypted_path).ok();
        fs::remove_file(decrypted_path).ok();
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    fs::write(output_path, ciphertext)?;
    println!("Encryption successful. Key: {}", hex::encode(key));
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key_bytes = hex::decode(key_hex)?;
    let key = key_bytes.as_slice().try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let ciphertext = fs::read(input_path)?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    println!("Decryption successful.");
    Ok(())
}