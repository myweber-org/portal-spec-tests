use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();
    
    println!("Enter data (press Ctrl+D when finished):");
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            input.push_str(&line);
            input.push('\n');
        }
    }
    
    let cleaned = clean_data(&input);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")
        .expect("Failed to create output file");
    
    output_file.write_all(cleaned.as_bytes())
        .expect("Failed to write to output file");
    
    println!("Data cleaned and saved to cleaned_output.txt");
}