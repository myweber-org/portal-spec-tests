use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file(path: &Path, key: &[u8; 32]) -> io::Result<EncryptionResult> {
    let plaintext = fs::read(path)?;
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12]).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_file(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> io::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill(&mut key);
    key
}

pub fn save_encrypted_data(output_path: &Path, result: &EncryptionResult) -> io::Result<()> {
    let mut file = fs::File::create(output_path)?;
    file.write_all(&result.nonce)?;
    file.write_all(&result.ciphertext)?;
    Ok(())
}

pub fn load_encrypted_data(path: &Path) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let data = fs::read(path)?;
    if data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce"
        ));
    }
    
    let nonce = data[..12].to_vec();
    let ciphertext = data[12..].to_vec();
    
    Ok((nonce, ciphertext))
}