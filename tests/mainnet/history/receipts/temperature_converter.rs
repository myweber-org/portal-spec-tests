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
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

pub fn fahrenheit_to_kelvin(fahrenheit: f64) -> f64 {
    celsius_to_kelvin(fahrenheit_to_celsius(fahrenheit))
}

pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

pub fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    celsius_to_fahrenheit(kelvin_to_celsius(kelvin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < f64::EPSILON);
        assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_celsius_to_kelvin() {
        assert!((celsius_to_kelvin(0.0) - 273.15).abs() < f64::EPSILON);
        assert!((celsius_to_kelvin(-273.15) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < f64::EPSILON);
        assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kelvin_conversions() {
        let kelvin = 300.0;
        let celsius = kelvin_to_celsius(kelvin);
        let fahrenheit = kelvin_to_fahrenheit(kelvin);
        
        assert!((celsius - 26.85).abs() < 0.01);
        assert!((fahrenheit - 80.33).abs() < 0.01);
    }
}