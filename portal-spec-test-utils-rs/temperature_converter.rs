fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{}°C is equal to {}°F", celsius_temp, fahrenheit_temp);
}fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{:.1}°C is equal to {:.1}°F", celsius_temp, fahrenheit_temp);

    let converted_back = fahrenheit_to_celsius(fahrenheit_temp);
    println!("{:.1}°F is equal to {:.1}°C", fahrenheit_temp, converted_back);
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
    fn test_kelvin_to_celsius() {
        assert!((kelvin_to_celsius(273.15) - 0.0).abs() < f64::EPSILON);
        assert!((kelvin_to_celsius(0.0) - (-273.15)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conversion_chain() {
        let celsius = 25.0;
        let fahrenheit = celsius_to_fahrenheit(celsius);
        let kelvin = fahrenheit_to_kelvin(fahrenheit);
        let celsius_back = kelvin_to_celsius(kelvin);
        assert!((celsius - celsius_back).abs() < f64::EPSILON * 100.0);
    }
}
use std::io;

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
    
    match choice {
        1 => {
            println!("Enter temperature in Celsius:");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            
            let celsius: f64 = match input.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid temperature value.");
                    return;
                }
            };
            
            let fahrenheit = celsius_to_fahrenheit(celsius);
            println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
        }
        2 => {
            println!("Enter temperature in Fahrenheit:");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            
            let fahrenheit: f64 = match input.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid temperature value.");
                    return;
                }
            };
            
            let celsius = fahrenheit_to_celsius(fahrenheit);
            println!("{:.2}°F = {:.2}°C", fahrenheit, celsius);
        }
        _ => println!("Invalid choice. Please select 1 or 2."),
    }
}
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temperatures = [0.0, 20.0, 37.0, 100.0];
    
    for &temp in &celsius_temperatures {
        let fahrenheit = celsius_to_fahrenheit(temp);
        println!("{}°C = {:.1}°F", temp, fahrenheit);
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

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    }

    #[test]
    fn test_kelvin_to_celsius() {
        assert_eq!(kelvin_to_celsius(273.15), 0.0);
        assert_eq!(kelvin_to_celsius(373.15), 100.0);
        assert_eq!(kelvin_to_celsius(0.0), -273.15);
    }

    #[test]
    fn test_conversion_chain() {
        let celsius = 25.0;
        let fahrenheit = celsius_to_fahrenheit(celsius);
        let kelvin = fahrenheit_to_kelvin(fahrenheit);
        let celsius_back = kelvin_to_celsius(kelvin);
        
        assert!((celsius - celsius_back).abs() < 1e-10);
    }
}
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
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
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    }
}