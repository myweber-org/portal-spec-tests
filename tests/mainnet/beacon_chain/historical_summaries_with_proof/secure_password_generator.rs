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
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || password.len() >= self.length - 2 {
                password.push(ch);
            }
        }

        Ok(password)
    }

    pub fn validate_strength(&self, password: &str) -> bool {
        if password.len() < self.length {
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
            } else if "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(ch) {
                has_special = true;
            }
        }

        (!self.use_lowercase || has_lowercase) &&
        (!self.use_uppercase || has_uppercase) &&
        (!self.use_digits || has_digit) &&
        (!self.use_special || has_special)
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
        assert!(generator.validate_strength(&password));
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .with_special(false)
            .with_uppercase(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(10)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        assert!(generator.generate().is_err());
    }
}