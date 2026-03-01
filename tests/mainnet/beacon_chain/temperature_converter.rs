
use std::io;

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn fahrenheit_to_kelvin(fahrenheit: f64) -> f64 {
    celsius_to_kelvin(fahrenheit_to_celsius(fahrenheit))
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    celsius_to_fahrenheit(kelvin_to_celsius(kelvin))
}

fn convert_temperature(value: f64, from: TemperatureUnit, to: TemperatureUnit) -> f64 {
    match (from, to) {
        (TemperatureUnit::Celsius, TemperatureUnit::Fahrenheit) => celsius_to_fahrenheit(value),
        (TemperatureUnit::Celsius, TemperatureUnit::Kelvin) => celsius_to_kelvin(value),
        (TemperatureUnit::Fahrenheit, TemperatureUnit::Celsius) => fahrenheit_to_celsius(value),
        (TemperatureUnit::Fahrenheit, TemperatureUnit::Kelvin) => fahrenheit_to_kelvin(value),
        (TemperatureUnit::Kelvin, TemperatureUnit::Celsius) => kelvin_to_celsius(value),
        (TemperatureUnit::Kelvin, TemperatureUnit::Fahrenheit) => kelvin_to_fahrenheit(value),
        _ => value,
    }
}

fn parse_unit(input: &str) -> Option<TemperatureUnit> {
    match input.to_lowercase().as_str() {
        "c" | "celsius" => Some(TemperatureUnit::Celsius),
        "f" | "fahrenheit" => Some(TemperatureUnit::Fahrenheit),
        "k" | "kelvin" => Some(TemperatureUnit::Kelvin),
        _ => None,
    }
}

fn main() {
    println!("Temperature Converter");
    println!("Supported units: Celsius (C), Fahrenheit (F), Kelvin (K)");

    loop {
        println!("\nEnter temperature value and source unit (e.g., '25 C'):");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.len() != 2 {
            println!("Invalid input format. Please enter value and unit separated by space.");
            continue;
        }
        
        let value: f64 = match parts[0].parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid temperature value.");
                continue;
            }
        };
        
        let from_unit = match parse_unit(parts[1]) {
            Some(unit) => unit,
            None => {
                println!("Invalid source unit. Use C, F, or K.");
                continue;
            }
        };
        
        println!("Enter target unit (C, F, or K):");
        let mut target_input = String::new();
        io::stdin().read_line(&mut target_input).expect("Failed to read line");
        
        let to_unit = match parse_unit(target_input.trim()) {
            Some(unit) => unit,
            None => {
                println!("Invalid target unit. Use C, F, or K.");
                continue;
            }
        };
        
        let result = convert_temperature(value, from_unit, to_unit);
        
        let from_str = match from_unit {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
            TemperatureUnit::Kelvin => "K",
        };
        
        let to_str = match to_unit {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
            TemperatureUnit::Kelvin => "K",
        };
        
        println!("{:.2} {} = {:.2} {}", value, from_str, result, to_str);
        
        println!("\nConvert another temperature? (y/n):");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        
        if choice.trim().to_lowercase() != "y" {
            break;
        }
    }
}