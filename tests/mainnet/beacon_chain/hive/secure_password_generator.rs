use rand::Rng;
use std::io;

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected!");
        return;
    }
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    print_strength_meter(&password);
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-128):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(len) if len >= 8 && len <= 128 => return len,
            Ok(len) => println!("Length must be between 8 and 128 (got {})", len),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut sets = Vec::new();
    let options = [
        ("Uppercase letters", UPPER.to_string()),
        ("Lowercase letters", LOWER.to_string()),
        ("Digits", DIGITS.to_string()),
        ("Special characters", SPECIAL.to_string()),
    ];
    
    println!("\nSelect character sets (enter numbers separated by spaces):");
    for (i, (name, _)) in options.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    for num_str in input.split_whitespace() {
        if let Ok(num) = num_str.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                sets.push(options[num - 1].1.clone());
            }
        }
    }
    
    sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let all_chars: String = char_sets.concat();
    
    // Ensure at least one character from each selected set
    let mut password_chars: Vec<char> = Vec::new();
    
    for set in char_sets {
        let idx = rng.gen_range(0..set.len());
        password_chars.push(set.chars().nth(idx).unwrap());
    }
    
    // Fill remaining length with random characters from all sets
    while password_chars.len() < length {
        let idx = rng.gen_range(0..all_chars.len());
        password_chars.push(all_chars.chars().nth(idx).unwrap());
    }
    
    // Shuffle the characters
    for i in 0..password_chars.len() {
        let j = rng.gen_range(0..password_chars.len());
        password_chars.swap(i, j);
    }
    
    password_chars.into_iter().collect()
}

fn print_strength_meter(password: &str) {
    let mut score = 0;
    
    if password.len() >= 12 { score += 2; }
    else if password.len() >= 8 { score += 1; }
    
    if password.chars().any(|c| UPPER.contains(c)) { score += 1; }
    if password.chars().any(|c| LOWER.contains(c)) { score += 1; }
    if password.chars().any(|c| DIGITS.contains(c)) { score += 1; }
    if password.chars().any(|c| SPECIAL.contains(c)) { score += 1; }
    
    println!("\nPassword Strength: {}", match score {
        5..=6 => "Strong",
        3..=4 => "Medium",
        _ => "Weak",
    });
}