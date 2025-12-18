use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    let nonce = generate_nonce();
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    if data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_slice, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> aes_gcm::Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    for (i, byte) in password_bytes.iter().cycle().take(32).enumerate() {
        key[i] = byte.wrapping_add(i as u8);
    }
    *aes_gcm::Key::<Aes256Gcm>::from_slice(&key)
}

fn generate_nonce() -> Nonce {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).clone()
}