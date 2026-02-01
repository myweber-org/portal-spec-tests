use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");

    let length = get_password_length();
    let char_sets = select_character_sets();

    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected.");
        return;
    }

    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", assess_password_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-64):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 64 => return length,
            Ok(_) => println!("Length must be between 8 and 64 characters."),
            Err(_) => println!("Please enter a valid number."),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut rng = rand::thread_rng();

    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Digits (0-9)");
    println!("4. Symbols (!@#$% etc.)");

    let choices = vec![
        (UPPERCASE.to_string(), "Uppercase"),
        (LOWERCASE.to_string(), "Lowercase"),
        (DIGITS.to_string(), "Digits"),
        (SYMBOLS.to_string(), "Symbols"),
    ];

    for (i, (_, name)) in choices.iter().enumerate() {
        let include = rng.gen_bool(0.7);
        if include {
            char_sets.push(choices[i].0.clone());
            println!("✓ {} - Included", name);
        } else {
            println!("✗ {} - Excluded", name);
        }
    }

    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let all_chars: String = char_sets.concat();
    let mut password = String::with_capacity(length);

    for _ in 0..length {
        let idx = rng.gen_range(0..all_chars.len());
        password.push(all_chars.chars().nth(idx).unwrap());
    }

    password
}

fn assess_password_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());

    let mut score = 0;
    if length >= 12 { score += 2; } else if length >= 8 { score += 1; }
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }

    match score {
        0..=2 => "Weak",
        3..=4 => "Moderate",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;

#[derive(Debug, Clone)]
pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_uppercase && !self.use_lowercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
        }

        let mut character_pool = Vec::new();
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()-_=+[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("Character pool is empty");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        Ok(password)
    }
}

fn main() -> io::Result<()> {
    println!("Secure Password Generator");
    println!("=========================");

    let generator = PasswordGenerator::new()
        .length(20)
        .uppercase(true)
        .lowercase(true)
        .digits(true)
        .special(true);

    match generator.generate() {
        Ok(password) => {
            println!("Generated password: {}", password);
            println!("Password length: {}", password.len());
        }
        Err(e) => {
            eprintln!("Error generating password: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let generator = PasswordGenerator::new();
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new().length(32);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 32);
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new()
            .uppercase(false)
            .lowercase(false)
            .digits(false)
            .special(false);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new().length(0);
        let result = generator.generate();
        assert!(result.is_err());
    }
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> Result<String, &'static str> {
    let mut character_pool = String::new();
    
    if config.use_uppercase { character_pool.push_str(UPPERCASE); }
    if config.use_lowercase { character_pool.push_str(LOWERCASE); }
    if config.use_digits { character_pool.push_str(DIGITS); }
    if config.use_special { character_pool.push_str(SPECIAL); }
    
    if character_pool.is_empty() {
        return Err("At least one character set must be selected");
    }
    
    if config.length == 0 {
        return Err("Password length must be greater than 0");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..character_pool.len());
            character_pool.chars().nth(idx).unwrap()
        })
        .collect();
    
    Ok(password)
}

fn get_user_input() -> PasswordConfig {
    let mut config = PasswordConfig::default();
    
    println!("Password Generator");
    println!("==================");
    
    let mut input = String::new();
    
    println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
    io::stdin().read_line(&mut input).unwrap();
    if let Ok(length) = input.trim().parse::<usize>() {
        config.length = length;
    }
    input.clear();
    
    println!("Include uppercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    config.use_uppercase = !input.trim().eq_ignore_ascii_case("n");
    input.clear();
    
    println!("Include lowercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    config.use_lowercase = !input.trim().eq_ignore_ascii_case("n");
    input.clear();
    
    println!("Include digits? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    config.use_digits = !input.trim().eq_ignore_ascii_case("n");
    input.clear();
    
    println!("Include special characters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    config.use_special = !input.trim().eq_ignore_ascii_case("n");
    
    config
}

fn main() {
    let config = get_user_input();
    
    match generate_password(&config) {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password length: {}", password.len());
            
            let mut has_uppercase = false;
            let mut has_lowercase = false;
            let mut has_digit = false;
            let mut has_special = false;
            
            for ch in password.chars() {
                if UPPERCASE.contains(ch) { has_uppercase = true; }
                if LOWERCASE.contains(ch) { has_lowercase = true; }
                if DIGITS.contains(ch) { has_digit = true; }
                if SPECIAL.contains(ch) { has_special = true; }
            }
            
            println!("\nCharacter set analysis:");
            println!("Uppercase letters: {}", if has_uppercase { "✓" } else { "✗" });
            println!("Lowercase letters: {}", if has_lowercase { "✓" } else { "✗" });
            println!("Digits: {}", if has_digit { "✓" } else { "✗" });
            println!("Special characters: {}", if has_special { "✓" } else { "✗" });
        }
        Err(e) => println!("Error: {}", e),
    }
}