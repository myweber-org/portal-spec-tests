use rand::Rng;
use std::io;

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected!");
        return;
    }
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    print_strength_meter(&password);
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-128):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(len) if len >= 8 && len <= 128 => return len,
            Ok(len) => println!("Length must be between 8 and 128 (got {})", len),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut sets = Vec::new();
    let options = [
        ("Uppercase letters", UPPER.to_string()),
        ("Lowercase letters", LOWER.to_string()),
        ("Digits", DIGITS.to_string()),
        ("Special characters", SPECIAL.to_string()),
    ];
    
    println!("\nSelect character sets (enter numbers separated by spaces):");
    for (i, (name, _)) in options.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    for num_str in input.split_whitespace() {
        if let Ok(num) = num_str.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                sets.push(options[num - 1].1.clone());
            }
        }
    }
    
    sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let all_chars: String = char_sets.concat();
    
    // Ensure at least one character from each selected set
    let mut password_chars: Vec<char> = Vec::new();
    
    for set in char_sets {
        let idx = rng.gen_range(0..set.len());
        password_chars.push(set.chars().nth(idx).unwrap());
    }
    
    // Fill remaining length with random characters from all sets
    while password_chars.len() < length {
        let idx = rng.gen_range(0..all_chars.len());
        password_chars.push(all_chars.chars().nth(idx).unwrap());
    }
    
    // Shuffle the characters
    for i in 0..password_chars.len() {
        let j = rng.gen_range(0..password_chars.len());
        password_chars.swap(i, j);
    }
    
    password_chars.into_iter().collect()
}

fn print_strength_meter(password: &str) {
    let mut score = 0;
    
    if password.len() >= 12 { score += 2; }
    else if password.len() >= 8 { score += 1; }
    
    if password.chars().any(|c| UPPER.contains(c)) { score += 1; }
    if password.chars().any(|c| LOWER.contains(c)) { score += 1; }
    if password.chars().any(|c| DIGITS.contains(c)) { score += 1; }
    if password.chars().any(|c| SPECIAL.contains(c)) { score += 1; }
    
    println!("\nPassword Strength: {}", match score {
        5..=6 => "Strong",
        3..=4 => "Medium",
        _ => "Weak",
    });
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    fn new() -> Self {
        PasswordGenerator {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    fn with_uppercase(mut self, enabled: bool) -> Self {
        self.use_uppercase = enabled;
        self
    }

    fn with_lowercase(mut self, enabled: bool) -> Self {
        self.use_lowercase = enabled;
        self
    }

    fn with_numbers(mut self, enabled: bool) -> Self {
        self.use_numbers = enabled;
        self
    }

    fn with_symbols(mut self, enabled: bool) -> Self {
        self.use_symbols = enabled;
        self
    }

    fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        let mut character_pool = String::new();
        
        if self.use_uppercase {
            character_pool.push_str(UPPERCASE);
        }
        if self.use_lowercase {
            character_pool.push_str(LOWERCASE);
        }
        if self.use_numbers {
            character_pool.push_str(NUMBERS);
        }
        if self.use_symbols {
            character_pool.push_str(SYMBOLS);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool.chars().nth(idx).unwrap()
            })
            .collect();

        Ok(password)
    }
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("=== Secure Password Generator ===");
    
    let length_input = get_user_input("Enter password length (default: 16):");
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };

    let uppercase_input = get_user_input("Include uppercase letters? (Y/n):");
    let use_uppercase = parse_bool_input(&uppercase_input);

    let lowercase_input = get_user_input("Include lowercase letters? (Y/n):");
    let use_lowercase = parse_bool_input(&lowercase_input);

    let numbers_input = get_user_input("Include numbers? (Y/n):");
    let use_numbers = parse_bool_input(&numbers_input);

    let symbols_input = get_user_input("Include symbols? (Y/n):");
    let use_symbols = parse_bool_input(&symbols_input);

    let generator = PasswordGenerator::new()
        .with_length(length)
        .with_uppercase(use_uppercase)
        .with_lowercase(use_lowercase)
        .with_numbers(use_numbers)
        .with_symbols(use_symbols);

    match generator.generate() {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Length: {}", password.len());
            
            let mut strength = "Weak";
            if password.len() >= 12 && use_uppercase && use_lowercase && use_numbers && use_symbols {
                strength = "Strong";
            } else if password.len() >= 8 {
                strength = "Medium";
            }
            
            println!("Estimated Strength: {}", strength);
        }
        Err(e) => println!("Error: {}", e),
    }
}