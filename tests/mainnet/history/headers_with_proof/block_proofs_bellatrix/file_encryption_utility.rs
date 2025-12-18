
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }

        let nonce = Nonce::from_slice(&ciphertext[0..12]);
        let ciphertext_data = &ciphertext[12..];

        self.cipher
            .decrypt(nonce, ciphertext_data)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

pub fn process_file_encryption(
    input_data: &[u8],
    operation: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let encryptor = FileEncryptor::new()?;

    match operation {
        "encrypt" => encryptor.encrypt_data(input_data),
        "decrypt" => encryptor.decrypt_data(input_data),
        _ => Err("Invalid operation. Use 'encrypt' or 'decrypt'".into()),
    }
}