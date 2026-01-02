use rand::Rng;

pub struct KeyGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl KeyGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: false,
        }
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
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
            return Err("Key length must be greater than zero");
        }

        let mut character_set = Vec::new();
        
        if self.use_uppercase {
            character_set.extend(b'A'..=b'Z');
        }
        
        if self.use_lowercase {
            character_set.extend(b'a'..=b'z');
        }
        
        if self.use_digits {
            character_set.extend(b'0'..=b'9');
        }
        
        if self.use_special {
            character_set.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_set.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut result = String::with_capacity(self.length);

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_set.len());
            result.push(character_set[idx] as char);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generation() {
        let generator = KeyGenerator::new(16);
        let key = generator.generate().unwrap();
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_custom_length() {
        let generator = KeyGenerator::new(32);
        let key = generator.generate().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_numeric_only() {
        let generator = KeyGenerator::new(8)
            .with_uppercase(false)
            .with_lowercase(false)
            .with_special(false);
        
        let key = generator.generate().unwrap();
        assert!(key.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_no_character_set() {
        let generator = KeyGenerator::new(8)
            .with_uppercase(false)
            .with_lowercase(false)
            .with_digits(false)
            .with_special(false);
        
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = KeyGenerator::new(0);
        assert!(generator.generate().is_err());
    }
}