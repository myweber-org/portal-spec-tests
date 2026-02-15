
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
    fn test_basic_password_generation() {
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
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit(10)));
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");
    
    let length = get_password_length();
    let char_set = select_character_set();
    
    let password = generate_password(length, &char_set);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", assess_password_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (default: {}): ", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
            
        let input = input.trim();
        
        if input.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match input.parse::<usize>() {
            Ok(length) if length >= 8 && length <= 128 => return length,
            Ok(_) => println!("Password length must be between 8 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_set() -> String {
    let mut char_set = String::new();
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Numbers (0-9)");
    println!("4. Symbols (!@#$% etc.)");
    println!("5. All of the above (default)");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
        
    let choice = input.trim();
    
    if choice == "1" || choice == "2" || choice == "3" || choice == "4" {
        if choice.contains('1') || choice == "5" || choice.is_empty() {
            char_set.push_str(UPPERCASE);
        }
        if choice.contains('2') || choice == "5" || choice.is_empty() {
            char_set.push_str(LOWERCASE);
        }
        if choice.contains('3') || choice == "5" || choice.is_empty() {
            char_set.push_str(NUMBERS);
        }
        if choice.contains('4') || choice == "5" || choice.is_empty() {
            char_set.push_str(SYMBOLS);
        }
    } else {
        char_set.push_str(UPPERCASE);
        char_set.push_str(LOWERCASE);
        char_set.push_str(NUMBERS);
        char_set.push_str(SYMBOLS);
    }
    
    if char_set.is_empty() {
        char_set.push_str(UPPERCASE);
        char_set.push_str(LOWERCASE);
        char_set.push_str(NUMBERS);
    }
    
    char_set
}

fn generate_password(length: usize, char_set: &str) -> String {
    let mut rng = rand::thread_rng();
    let char_set_bytes = char_set.as_bytes();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..char_set_bytes.len());
            char_set_bytes[idx] as char
        })
        .collect()
}

fn assess_password_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = 0;
    
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
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

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
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
            return Err("Password length must be greater than zero");
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
    fn test_password_length() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit(10)));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }
}