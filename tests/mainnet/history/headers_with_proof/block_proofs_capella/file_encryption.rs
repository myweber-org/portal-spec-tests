
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key.as_bytes());

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_file_processing() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let test_data = b"test content for encryption";
        let key = "test_key";

        input_file.write_all(test_data)?;

        process_file(input_file.path(), output_file.path(), key)?;

        let mut processed = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut processed)?;

        let mut expected = test_data.to_vec();
        xor_cipher(&mut expected, key.as_bytes());

        assert_eq!(processed, expected);
        Ok(())
    }
}