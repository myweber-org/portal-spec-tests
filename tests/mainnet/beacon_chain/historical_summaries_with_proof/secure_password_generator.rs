use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
    
    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12, true, true, true);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_lowercase_only() {
        let password = generate_password(8, false, false, false);
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_contains_uppercase() {
        let password = generate_password(20, true, false, false);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
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
        Self {
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
            return Err("Password length must be greater than 0");
        }

        if !self.use_uppercase && !self.use_lowercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
        }

        let mut character_pool = Vec::new();
        let mut required_chars = HashSet::new();

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.insert(self.random_uppercase());
        }

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.insert(self.random_lowercase());
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.insert(self.random_digit());
        }

        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
            required_chars.insert(self.random_special());
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = required_chars.into_iter().collect();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        for i in (1..password_chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            password_chars.swap(i, j);
        }

        Ok(password_chars.into_iter().collect())
    }

    fn random_uppercase(&self) -> char {
        let mut rng = rand::thread_rng();
        (rng.gen_range(b'A'..=b'Z')) as char
    }

    fn random_lowercase(&self) -> char {
        let mut rng = rand::thread_rng();
        (rng.gen_range(b'a'..=b'z')) as char
    }

    fn random_digit(&self) -> char {
        let mut rng = rand::thread_rng();
        (rng.gen_range(b'0'..=b'9')) as char
    }

    fn random_special(&self) -> char {
        let mut rng = rand::thread_rng();
        let special_ranges = [
            (b'!', b'/'),
            (b':', b'@'),
            (b'[', b'`'),
            (b'{', b'~'),
        ];
        let range_idx = rng.gen_range(0..special_ranges.len());
        let (start, end) = special_ranges[range_idx];
        (rng.gen_range(start..=end)) as char
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
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit()));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .lowercase(false)
            .digits(false)
            .special(false);
        assert!(generator.generate().is_err());
    }
}