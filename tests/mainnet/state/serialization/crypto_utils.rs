use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    let password: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    password
}

pub fn generate_secure_token() -> String {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    token
}