
use std::io::{self, Write};

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn main() {
    println!("Temperature Converter");
    println!("1. Celsius to Fahrenheit");
    println!("2. Fahrenheit to Celsius");

    let choice: u32 = loop {
        print!("Select conversion (1 or 2): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse() {
            Ok(num) if num == 1 || num == 2 => break num,
            _ => println!("Invalid input. Please enter 1 or 2."),
        }
    };

    let temperature: f64 = loop {
        print!("Enter temperature: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse() {
            Ok(num) => break num,
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    };

    let result = match choice {
        1 => celsius_to_fahrenheit(temperature),
        2 => fahrenheit_to_celsius(temperature),
        _ => unreachable!(),
    };

    let (from_unit, to_unit) = match choice {
        1 => ("Celsius", "Fahrenheit"),
        2 => ("Fahrenheit", "Celsius"),
        _ => unreachable!(),
    };

    println!("{:.2}° {} = {:.2}° {}", temperature, from_unit, result, to_unit);
}