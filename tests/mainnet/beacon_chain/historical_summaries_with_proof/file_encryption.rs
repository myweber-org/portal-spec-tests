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

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            output_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
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

        let original_data = b"Hello, this is a test message for XOR encryption!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        cipher.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_data)
            .unwrap();

        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XORCipher::new("");
        let data = b"test data";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let mut processed_data = Vec::new();
        fs::File::open(output_file.path())
            .unwrap()
            .read_to_end(&mut processed_data)
            .unwrap();

        assert_eq!(data.to_vec(), processed_data);
    }
}