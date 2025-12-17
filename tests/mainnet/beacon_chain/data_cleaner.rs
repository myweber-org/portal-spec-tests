use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().copied().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    println!("Enter data (press Ctrl+D when finished):");
    for line in stdin.lock().lines() {
        match line {
            Ok(content) => buffer.push_str(&content),
            Err(_) => break,
        }
    }
    
    let cleaned = clean_data(&buffer);
    println!("Cleaned data:");
    println!("{}", cleaned);
}