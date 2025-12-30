
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    thread_rng().fill(&mut token);
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(16);
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_token_size() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
    }
}