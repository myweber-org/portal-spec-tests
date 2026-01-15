use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encrypt() {
        let data = b"Hello, World!";
        let key = b"secret";
        let encrypted = xor_encrypt(data, key);
        let decrypted = xor_encrypt(&encrypted, key);
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_process_file() -> io::Result<()> {
        let test_data = b"Test file content";
        let key = b"testkey";

        fs::write("test_input.txt", test_data)?;
        process_file("test_input.txt", "test_output.txt", key)?;

        let encrypted = fs::read("test_output.txt")?;
        let decrypted = xor_encrypt(&encrypted, key);
        assert_eq!(test_data.to_vec(), decrypted);

        fs::remove_file("test_input.txt")?;
        fs::remove_file("test_output.txt")?;
        Ok(())
    }
}