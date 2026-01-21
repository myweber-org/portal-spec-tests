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
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("test_password").unwrap();
        let test_data = b"Hello, encrypted world!";
        
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
        assert_eq!(decrypted_data, test_data);
    }
}use std::fs;
use std::io::{self, Read, Write};

const XOR_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte ^= XOR_KEY;
    }
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_file(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

fn base64_decode(encoded: &str) -> io::Result<Vec<u8>> {
    base64::decode(encoded).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut data = read_file(input_path)?;
    xor_cipher(&mut data);
    let encoded = base64_encode(&data);
    write_file(output_path, encoded.as_bytes())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let encoded = read_file(input_path)?;
    let encoded_str = String::from_utf8_lossy(&encoded);
    let mut data = base64_decode(&encoded_str)?;
    xor_cipher(&mut data);
    write_file(output_path, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let original = b"Hello, World! This is a test message.";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original).unwrap();

        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        encrypt_file(temp_input.path().to_str().unwrap(), 
                    temp_encrypted.path().to_str().unwrap()).unwrap();
        decrypt_file(temp_encrypted.path().to_str().unwrap(), 
                    temp_decrypted.path().to_str().unwrap()).unwrap();

        let decrypted = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        xor_cipher(&mut data);
        assert_eq!(data, vec![0xAA, 0x55, 0xFF, 0x00]);
        xor_cipher(&mut data);
        assert_eq!(data, vec![0x00, 0xFF, 0x55, 0xAA]);
    }
}