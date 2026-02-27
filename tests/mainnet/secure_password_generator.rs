
use rand::Rng;
use std::collections::HashSet;

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
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            password.push(ch);
            used_chars.insert(ch);
        }

        if used_chars.len() < (self.length / 2).max(1) {
            return Err("Generated password lacks sufficient character variety");
        }

        Ok(password)
    }
}

pub fn validate_password_strength(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }

    let mut has_lowercase = false;
    let mut has_uppercase = false;
    let mut has_digit = false;
    let mut has_special = false;

    for ch in password.chars() {
        if ch.is_ascii_lowercase() {
            has_lowercase = true;
        } else if ch.is_ascii_uppercase() {
            has_uppercase = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if ch.is_ascii_punctuation() {
            has_special = true;
        }
    }

    has_lowercase && has_uppercase && has_digit && has_special
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
        assert!(validate_password_strength(&password));
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_validation() {
        assert!(!validate_password_strength("weak"));
        assert!(!validate_password_strength("weakpass"));
        assert!(validate_password_strength("StrongP@ss1"));
    }
}use rand::Rng;
use std::collections::HashSet;

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
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        let mut required_sets = Vec::new();

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_sets.push(b'a'..=b'z');
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_sets.push(b'A'..=b'Z');
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_sets.push(b'0'..=b'9');
        }

        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
            required_sets.push(b'!'..=b'/');
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_sets = HashSet::new();

        for i in 0..self.length {
            if i < required_sets.len() {
                let set = &required_sets[i];
                let random_char = rng.gen_range(set.start..=set.end);
                password_chars.push(random_char as char);
                used_sets.insert(i);
            } else {
                let random_index = rng.gen_range(0..character_pool.len());
                password_chars.push(character_pool[random_index] as char);
            }
        }

        for (i, set) in required_sets.iter().enumerate() {
            if !used_sets.contains(&i) {
                let random_char = rng.gen_range(set.start..=set.end);
                let replace_pos = rng.gen_range(0..self.length);
                password_chars[replace_pos] = random_char as char;
            }
        }

        for _ in 0..self.length * 2 {
            let pos1 = rng.gen_range(0..self.length);
            let pos2 = rng.gen_range(0..self.length);
            password_chars.swap(pos1, pos2);
        }

        Ok(password_chars.into_iter().collect())
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
    fn test_custom_length() {
        let generator = PasswordGenerator::new(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_only_digits() {
        let generator = PasswordGenerator::new(8)
            .lowercase(false)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
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
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    fn new() -> Self {
        PasswordGenerator {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    fn with_numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    fn with_symbols(mut self, enable: bool) -> Self {
        self.use_symbols = enable;
        self
    }

    fn generate(&self) -> Result<String, &'static str> {
        let mut character_pool = String::new();
        
        if self.use_uppercase {
            character_pool.push_str(UPPERCASE);
        }
        if self.use_lowercase {
            character_pool.push_str(LOWERCASE);
        }
        if self.use_numbers {
            character_pool.push_str(NUMBERS);
        }
        if self.use_symbols {
            character_pool.push_str(SYMBOLS);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        if self.length == 0 {
            return Err("Password length must be greater than zero");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool.chars().nth(idx).unwrap()
            })
            .collect();

        Ok(password)
    }
}

fn main() {
    println!("Secure Password Generator");
    println!("=========================");

    let mut input = String::new();
    
    println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
    io::stdin().read_line(&mut input).unwrap();
    let length: usize = input.trim().parse().unwrap_or(DEFAULT_LENGTH);
    
    input.clear();
    println!("Include uppercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let use_uppercase = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include lowercase letters? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let use_lowercase = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include numbers? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let use_numbers = !input.trim().eq_ignore_ascii_case("n");
    
    input.clear();
    println!("Include symbols? (y/n, default: y): ");
    io::stdin().read_line(&mut input).unwrap();
    let use_symbols = !input.trim().eq_ignore_ascii_case("n");

    let generator = PasswordGenerator::new()
        .with_length(length)
        .with_uppercase(use_uppercase)
        .with_lowercase(use_lowercase)
        .with_numbers(use_numbers)
        .with_symbols(use_symbols);

    match generator.generate() {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Strength: {}", evaluate_strength(&password));
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn evaluate_strength(password: &str) -> &'static str {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());

    let mut score = 0;
    if length >= 12 { score += 1; }
    if length >= 16 { score += 1; }
    if has_upper && has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }

    match score {
        0..=1 => "Weak",
        2..=3 => "Moderate",
        4 => "Strong",
        _ => "Very Strong",
    }
}