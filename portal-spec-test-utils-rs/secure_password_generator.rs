use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::HashSet;

const PASSWORD_LENGTH: usize = 16;
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SPECIALS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

pub struct PasswordGenerator {
    character_sets: Vec<Vec<u8>>,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        PasswordGenerator {
            character_sets: vec![
                UPPERCASE.to_vec(),
                LOWERCASE.to_vec(),
                DIGITS.to_vec(),
                SPECIALS.to_vec(),
            ],
        }
    }

    pub fn generate(&self) -> String {
        let mut rng = OsRng;
        let mut password_bytes = Vec::with_capacity(PASSWORD_LENGTH);
        let mut used_indices = HashSet::new();

        for set in &self.character_sets {
            let mut index;
            loop {
                index = (rng.next_u32() % set.len() as u32) as usize;
                if !used_indices.contains(&index) {
                    used_indices.insert(index);
                    break;
                }
            }
            password_bytes.push(set[index]);
        }

        while password_bytes.len() < PASSWORD_LENGTH {
            let set_index = (rng.next_u32() % self.character_sets.len() as u32) as usize;
            let char_index = (rng.next_u32() % self.character_sets[set_index].len() as u32) as usize;
            password_bytes.push(self.character_sets[set_index][char_index]);
        }

        for i in (1..PASSWORD_LENGTH).rev() {
            let j = (rng.next_u32() % (i as u32 + 1)) as usize;
            password_bytes.swap(i, j);
        }

        String::from_utf8(password_bytes).expect("Generated password should be valid UTF-8")
    }

    pub fn validate(&self, password: &str) -> bool {
        if password.len() != PASSWORD_LENGTH {
            return false;
        }

        let bytes = password.as_bytes();
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;

        for &byte in bytes {
            if UPPERCASE.contains(&byte) {
                has_upper = true;
            } else if LOWERCASE.contains(&byte) {
                has_lower = true;
            } else if DIGITS.contains(&byte) {
                has_digit = true;
            } else if SPECIALS.contains(&byte) {
                has_special = true;
            }
        }

        has_upper && has_lower && has_digit && has_special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new();
        let password = generator.generate();
        
        assert_eq!(password.len(), PASSWORD_LENGTH);
        assert!(generator.validate(&password));
    }

    #[test]
    fn test_password_validation() {
        let generator = PasswordGenerator::new();
        
        assert!(!generator.validate("short"));
        assert!(!generator.validate("nouppercase123!"));
        assert!(!generator.validate("NOLOWERCASE123!"));
        assert!(!generator.validate("NoDigitsHere!"));
        assert!(!generator.validate("NoSpecial12345"));
    }

    #[test]
    fn test_unique_passwords() {
        let generator = PasswordGenerator::new();
        let mut passwords = HashSet::new();
        
        for _ in 0..100 {
            let password = generator.generate();
            assert!(passwords.insert(password));
        }
    }
}
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
            return Err("Generated password lacks sufficient character diversity");
        }

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
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit(10)));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }
}use rand::Rng;
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
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
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        let mut required_chars = HashSet::new();

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.insert(b'A');
        }
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.insert(b'a');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.insert(b'0');
        }
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
            required_chars.insert(b'!');
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx];
            password_chars.push(ch);
            used_chars.insert(ch);
        }

        for &required in &required_chars {
            if !used_chars.contains(&required) {
                let replace_pos = rng.gen_range(0..self.length);
                password_chars[replace_pos] = required;
            }
        }

        Ok(String::from_utf8(password_chars).unwrap())
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
            .uppercase(true)
            .lowercase(false)
            .digits(true)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(!password.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_invalid_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
    }
}
use rand::Rng;
use std::collections::HashSet;

#[derive(Debug, Clone)]
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

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn with_special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, String> {
        if self.length == 0 {
            return Err("Password length must be greater than 0".to_string());
        }

        let character_sets = self.build_character_sets();
        if character_sets.is_empty() {
            return Err("At least one character set must be enabled".to_string());
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_sets = HashSet::new();

        for i in 0..self.length {
            let set_index = rng.gen_range(0..character_sets.len());
            let charset = &character_sets[set_index];
            let char_index = rng.gen_range(0..charset.len());
            
            password_chars.push(charset.chars().nth(char_index).unwrap());
            used_sets.insert(set_index);
        }

        if used_sets.len() < character_sets.len() {
            return self.generate();
        }

        Ok(password_chars.into_iter().collect())
    }

    fn build_character_sets(&self) -> Vec<String> {
        let mut sets = Vec::new();
        
        if self.use_lowercase {
            sets.push("abcdefghijklmnopqrstuvwxyz".to_string());
        }
        if self.use_uppercase {
            sets.push("ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string());
        }
        if self.use_digits {
            sets.push("0123456789".to_string());
        }
        if self.use_special {
            sets.push("!@#$%^&*()-_=+[]{}|;:,.<>?".to_string());
        }
        
        sets
    }

    pub fn calculate_entropy(&self) -> f64 {
        let mut pool_size = 0;
        
        if self.use_lowercase {
            pool_size += 26;
        }
        if self.use_uppercase {
            pool_size += 26;
        }
        if self.use_digits {
            pool_size += 10;
        }
        if self.use_special {
            pool_size += 32;
        }

        if pool_size == 0 {
            return 0.0;
        }

        (self.length as f64) * (pool_size as f64).log2()
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
        let generator = PasswordGenerator::new(10)
            .with_special(false)
            .with_digits(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_entropy_calculation() {
        let generator = PasswordGenerator::new(16);
        let entropy = generator.calculate_entropy();
        assert!(entropy > 80.0);
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
        
        let generator = PasswordGenerator::new(10)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        assert!(generator.generate().is_err());
    }
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

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn with_special(mut self, enable: bool) -> Self {
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
    fn test_generate_password() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .with_lowercase(true)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
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
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        let result = generator.generate();
        assert!(result.is_err());
    }
}