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