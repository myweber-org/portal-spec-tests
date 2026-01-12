
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

pub fn generate_numeric_code(digits: u32) -> u32 {
    let min = 10u32.pow(digits - 1);
    let max = 10u32.pow(digits) - 1;
    thread_rng().gen_range(min..=max)
}