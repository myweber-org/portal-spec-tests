
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

    pub fn generate(&self) -> Result<String, String> {
        if self.length == 0 {
            return Err("Password length must be greater than 0".to_string());
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
            character_pool.extend(b"!@#$%^&*()-_=+[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled".to_string());
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

        if self.validate_character_usage(&used_chars) {
            Ok(password)
        } else {
            Err("Generated password doesn't meet character set requirements".to_string())
        }
    }

    fn validate_character_usage(&self, used_chars: &HashSet<char>) -> bool {
        let has_lowercase = !self.use_lowercase || used_chars.iter().any(|c| c.is_lowercase());
        let has_uppercase = !self.use_uppercase || used_chars.iter().any(|c| c.is_uppercase());
        let has_digits = !self.use_digits || used_chars.iter().any(|c| c.is_digit(10));
        let has_special = !self.use_special || used_chars.iter().any(|c| {
            let special_chars = "!@#$%^&*()-_=+[]{}|;:,.<>?";
            special_chars.contains(*c)
        });

        has_lowercase && has_uppercase && has_digits && has_special
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
    fn test_invalid_length() {
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
}