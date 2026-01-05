use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";

    if use_uppercase {
        charset.push_str(uppercase);
    }
    if use_numbers {
        charset.push_str(numbers);
    }
    if use_symbols {
        charset.push_str(symbols);
    }

    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect();

    password
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
        let password = generate_password(10, false, false, false);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_contains_uppercase() {
        let password = generate_password(15, true, false, false);
        assert!(password.chars().any(|c| c.is_uppercase()));
    }
}