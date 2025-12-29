
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

fn xor_encrypt_decrypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_encrypt_decrypt(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    match process_file(input_file, output_file, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_symmetry() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        let original = data.clone();
        let key = 0x33;

        xor_encrypt_decrypt(&mut data, key);
        assert_ne!(data, original);

        xor_encrypt_decrypt(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_processing() {
        let test_data = b"Hello, XOR encryption!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        let final_path = "test_final.txt";

        fs::write(input_path, test_data).unwrap();

        process_file(input_path, output_path, DEFAULT_KEY).unwrap();
        process_file(output_path, final_path, DEFAULT_KEY).unwrap();

        let result = fs::read(final_path).unwrap();
        assert_eq!(result, test_data);

        fs::remove_file(input_path).ok();
        fs::remove_file(output_path).ok();
        fs::remove_file(final_path).ok();
    }
}