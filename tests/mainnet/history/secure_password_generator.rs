use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let length = get_password_length();
    let char_set = select_character_sets();
    
    if char_set.is_empty() {
        println!("Error: No character sets selected!");
        return;
    }
    
    let password = generate_password(length, &char_set);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-64):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 64 => return length,
            Ok(_) => println!("Length must be between 8 and 64"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> String {
    let mut char_set = String::new();
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters");
    println!("2. Lowercase letters");
    println!("3. Digits");
    println!("4. Special characters");
    println!("Enter numbers separated by spaces (e.g., '1 2 3 4'):");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    for num in input.split_whitespace() {
        match num {
            "1" => char_set.push_str(UPPERCASE),
            "2" => char_set.push_str(LOWERCASE),
            "3" => char_set.push_str(DIGITS),
            "4" => char_set.push_str(SPECIAL),
            _ => continue,
        }
    }
    
    char_set
}

fn generate_password(length: usize, char_set: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..char_set.len());
        password.push(char_set.chars().nth(idx).unwrap());
    }
    
    password
}

fn evaluate_strength(password: &str) -> String {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    let criteria_count = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();
    
    match criteria_count {
        4 if password.len() >= 16 => "Very Strong",
        4 => "Strong",
        3 => "Good",
        2 => "Weak",
        _ => "Very Weak",
    }.to_string()
}