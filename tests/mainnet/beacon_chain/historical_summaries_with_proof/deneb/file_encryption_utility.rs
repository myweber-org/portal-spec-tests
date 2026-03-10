use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match process_file(input_path, output_path, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;
        
        xor_cipher(&mut data, key);
        xor_cipher(&mut data, key);
        
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_processing() {
        let test_data = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        
        fs::write(input_path, test_data).unwrap();
        
        process_file(input_path, output_path, DEFAULT_KEY).unwrap();
        process_file(output_path, "test_decrypted.txt", DEFAULT_KEY).unwrap();
        
        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_data);
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
        fs::remove_file("test_decrypted.txt").unwrap();
    }
}