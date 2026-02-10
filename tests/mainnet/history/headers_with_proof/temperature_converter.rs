
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
        assert!((celsius_to_kelvin(100.0) - 373.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < f64::EPSILON);
        assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kelvin_to_celsius() {
        assert!((kelvin_to_celsius(273.15) - 0.0).abs() < f64::EPSILON);
        assert!((kelvin_to_celsius(373.15) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conversion_chain() {
        let celsius = 25.0;
        let fahrenheit = celsius_to_fahrenheit(celsius);
        let kelvin = celsius_to_kelvin(celsius);
        let celsius_back = fahrenheit_to_celsius(fahrenheit);
        let kelvin_back = fahrenheit_to_kelvin(fahrenheit);
        
        assert!((celsius - celsius_back).abs() < f64::EPSILON);
        assert!((kelvin - kelvin_back).abs() < f64::EPSILON);
    }
}use std::io;

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

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    let choice: u32 = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Please enter 1 or 2.");
            return;
        }
    };

    println!("Enter temperature value:");
    let mut temp_input = String::new();
    io::stdin()
        .read_line(&mut temp_input)
        .expect("Failed to read line");

    let temperature: f64 = match temp_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid temperature value.");
            return;
        }
    };

    match choice {
        1 => {
            let result = celsius_to_fahrenheit(temperature);
            println!("{:.2}°C = {:.2}°F", temperature, result);
        }
        2 => {
            let result = fahrenheit_to_celsius(temperature);
            println!("{:.2}°F = {:.2}°C", temperature, result);
        }
        _ => println!("Invalid choice. Please select 1 or 2."),
    }
}
use std::io;

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature in Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let celsius: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Please enter a valid number.");
            return;
        }
    };

    let fahrenheit = celsius_to_fahrenheit(celsius);
    let kelvin = celsius_to_kelvin(celsius);

    println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
    println!("{:.2}°C = {:.2}K", celsius, kelvin);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
        assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
    }

    #[test]
    fn test_celsius_to_kelvin() {
        assert_eq!(celsius_to_kelvin(0.0), 273.15);
        assert_eq!(celsius_to_kelvin(100.0), 373.15);
        assert_eq!(celsius_to_kelvin(-273.15), 0.0);
    }
}