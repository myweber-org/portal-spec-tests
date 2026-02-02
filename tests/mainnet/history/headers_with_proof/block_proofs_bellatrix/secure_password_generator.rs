use rand::Rng;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
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

        let mut character_pool = Vec::new();
        
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        
        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit()));
    }

    #[test]
    fn test_invalid_length() {
        let generator = PasswordGenerator::new(0);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
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

fn main() {
    println!("Secure Password Generator");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected");
        return;
    }
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
            
        let input = input.trim();
        
        if input.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match input.parse::<usize>() {
            Ok(length) if length >= 4 && length <= 128 => return length,
            Ok(_) => println!("Length must be between 4 and 128"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut rng = rand::thread_rng();
    
    println!("\nSelect character sets (at least one required):");
    println!("1. Uppercase letters");
    println!("2. Lowercase letters");
    println!("3. Digits");
    println!("4. Special characters");
    println!("5. All of the above");
    println!("Enter choices separated by spaces (e.g., '1 3 4'):");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
        
    let choices: Vec<&str> = input.trim().split_whitespace().collect();
    
    if choices.contains(&"5") {
        char_sets.push(UPPERCASE.to_string());
        char_sets.push(LOWERCASE.to_string());
        char_sets.push(DIGITS.to_string());
        char_sets.push(SPECIAL.to_string());
        return char_sets;
    }
    
    for choice in choices {
        match choice {
            "1" => char_sets.push(UPPERCASE.to_string()),
            "2" => char_sets.push(LOWERCASE.to_string()),
            "3" => char_sets.push(DIGITS.to_string()),
            "4" => char_sets.push(SPECIAL.to_string()),
            _ => continue,
        }
    }
    
    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let set_index = rng.gen_range(0..char_sets.len());
        let char_set = &char_sets[set_index];
        let char_index = rng.gen_range(0..char_set.len());
        
        if let Some(c) = char_set.chars().nth(char_index) {
            password.push(c);
        }
    }
    
    password
}

fn evaluate_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = 0;
    
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_special { score += 2; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Moderate",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}