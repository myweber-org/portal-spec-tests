use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str, salt: &[u8]) -> Self {
        let mut key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Self { cipher }
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_LENGTH] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output)
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, plaintext)
    }
}

pub fn generate_salt() -> [u8; SALT_LENGTH] {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn process_encryption(
    password: &str,
    input_file: &str,
    output_file: &str,
    encrypt: bool
) -> io::Result<()> {
    let salt = generate_salt();
    let encryptor = FileEncryptor::new(password, &salt);
    
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);
    
    if encrypt {
        encryptor.encrypt_file(input_path, output_path)?;
        println!("Encryption successful. Salt (hex): {}", hex::encode(salt));
    } else {
        print!("Enter salt (hex): ");
        io::stdout().flush()?;
        
        let mut salt_input = String::new();
        io::stdin().read_line(&mut salt_input)?;
        
        let salt_bytes = hex::decode(salt_input.trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        if salt_bytes.len() != SALT_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Salt must be {} bytes", SALT_LENGTH)
            ));
        }
        
        let encryptor = FileEncryptor::new(password, &salt_bytes);
        encryptor.decrypt_file(input_path, output_path)?;
        println!("Decryption successful");
    }
    
    Ok(())
}