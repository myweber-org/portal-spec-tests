
use std::fs;
use std::io::{self, Read, Write};

const KEY: u8 = 0xAA;

fn xor_encrypt_decrypt(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte ^= KEY;
    }
}

fn process_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_encrypt_decrypt(&mut buffer);

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

    let input_file = &args[1];
    let output_file = &args[2];

    match process_file(input_file, output_file) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
}