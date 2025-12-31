use std::io;

fn main() {
    println!("Temperature Converter");
    println!("Enter a temperature value followed by its unit (C or F), e.g., '25 C' or '77 F':");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 {
        println!("Invalid input format. Please provide a number and a unit (C or F).");
        return;
    }

    let value: f64 = match parts[0].parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid number.");
            return;
        }
    };

    let unit = parts[1].to_uppercase();
    match unit.as_str() {
        "C" => {
            let fahrenheit = (value * 9.0 / 5.0) + 32.0;
            println!("{:.2}°C is equal to {:.2}°F", value, fahrenheit);
        }
        "F" => {
            let celsius = (value - 32.0) * 5.0 / 9.0;
            println!("{:.2}°F is equal to {:.2}°C", value, celsius);
        }
        _ => println!("Invalid unit. Please use 'C' for Celsius or 'F' for Fahrenheit."),
    }
}