use std::fs::{self, File};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        let mut input_file = File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.process_bytes(&buffer);

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_length = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_length])
            .collect()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let cipher = XORCipher::new(key);

        let test_data = b"Hello, World!";
        let encrypted = cipher.process_bytes(test_data);
        let decrypted = cipher.process_bytes(&encrypted);

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key_123";
        let cipher = XORCipher::new(key);

        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "Test file content for encryption").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        cipher
            .encrypt_file(input_file.path(), output_file.path())
            .unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        cipher
            .decrypt_file(output_file.path(), decrypted_file.path())
            .unwrap();

        let mut decrypted_content = String::new();
        File::open(decrypted_file.path())
            .unwrap()
            .read_to_string(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, "Test file content for encryption");
    }
}