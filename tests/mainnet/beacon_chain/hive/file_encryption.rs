
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    if key_bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Encryption key cannot be empty",
        ));
    }

    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key_bytes);
    fs::write(output_path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret";
        let mut data = b"hello world".to_vec();
        let original = data.clone();

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key";
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        write!(input_file, "Sample file content").unwrap();

        process_file(input_file.path(), output_file.path(), key).unwrap();

        let encrypted = fs::read(output_file.path()).unwrap();
        assert_ne!(encrypted, b"Sample file content");

        let mut temp_file = NamedTempFile::new().unwrap();
        process_file(output_file.path(), temp_file.path(), key).unwrap();
        let decrypted = fs::read(temp_file.path()).unwrap();
        assert_eq!(decrypted, b"Sample file content");
    }
}