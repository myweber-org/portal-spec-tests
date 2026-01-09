
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| e.to_string())?;

        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&nonce).map_err(|e| e.to_string())?;
        output.write_all(&ciphertext).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("Invalid encrypted file format".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        fs::write(output_path, plaintext).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn generate_key_file(path: &Path) -> Result<(), String> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    fs::write(path, key.as_slice()).map_err(|e| e.to_string())?;
    Ok(())
}use std::fs::{self, File};
use std::io::{Read, Write};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), std::io::Error> {
        let mut input_file = File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_transform(&buffer);

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), std::io::Error> {
        self.encrypt_file(input_path, output_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

pub fn process_directory(
    cipher: &XORCipher,
    dir_path: &Path,
    output_dir: &Path,
    encrypt: bool,
) -> Result<(), std::io::Error> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let output_path = output_dir.join(file_name);

            if encrypt {
                cipher.encrypt_file(&path, &output_path)?;
                println!("Encrypted: {:?} -> {:?}", path, output_path);
            } else {
                cipher.decrypt_file(&path, &output_path)?;
                println!("Decrypted: {:?} -> {:?}", path, output_path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let test_data = b"Hello, XOR encryption!";

        let encrypted = cipher.xor_transform(test_data);
        let decrypted = cipher.xor_transform(&encrypted);

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> Result<(), std::io::Error> {
        let cipher = XORCipher::new("test_key");

        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(b"Test file content")?;

        let output_file = NamedTempFile::new()?;

        cipher.encrypt_file(input_file.path(), output_file.path())?;

        let mut encrypted_content = Vec::new();
        File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;

        assert_ne!(encrypted_content, b"Test file content");

        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;

        let mut decrypted_content = Vec::new();
        File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;

        assert_eq!(decrypted_content, b"Test file content");

        Ok(())
    }
}