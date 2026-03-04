
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
    fn test_fahrenheit_to_kelvin() {
        assert_eq!(fahrenheit_to_kelvin(32.0), 273.15);
        assert_eq!(fahrenheit_to_kelvin(212.0), 373.15);
    }

    #[test]
    fn test_kelvin_to_celsius() {
        assert_eq!(kelvin_to_celsius(273.15), 0.0);
        assert_eq!(kelvin_to_celsius(373.15), 100.0);
        assert_eq!(kelvin_to_celsius(0.0), -273.15);
    }

    #[test]
    fn test_kelvin_to_fahrenheit() {
        assert_eq!(kelvin_to_fahrenheit(273.15), 32.0);
        assert_eq!(kelvin_to_fahrenheit(373.15), 212.0);
    }
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

fn main() {
    let celsius_temp = 25.0;
    println!("{:.2}°C = {:.2}°F", celsius_temp, celsius_to_fahrenheit(celsius_temp));
    println!("{:.2}°C = {:.2}K", celsius_temp, celsius_to_kelvin(celsius_temp));
    
    let fahrenheit_temp = 77.0;
    println!("{:.2}°F = {:.2}°C", fahrenheit_temp, fahrenheit_to_celsius(fahrenheit_temp));
    println!("{:.2}°F = {:.2}K", fahrenheit_temp, fahrenheit_to_kelvin(fahrenheit_temp));
    
    let kelvin_temp = 298.15;
    println!("{:.2}K = {:.2}°C", kelvin_temp, kelvin_to_celsius(kelvin_temp));
    println!("{:.2}K = {:.2}°F", kelvin_temp, kelvin_to_fahrenheit(kelvin_temp));
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
        
        assert!((celsius - celsius_back).abs() < 0.0001);
    }
}fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{:.1}°C is equal to {:.1}°F", celsius_temp, fahrenheit_temp);
}use std::io;

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
            println!("Invalid input. Please enter a number.");
            return;
        }
    };

    let fahrenheit = celsius_to_fahrenheit(celsius);
    let kelvin = celsius_to_kelvin(celsius);

    println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
    println!("{:.2}°C = {:.2}K", celsius, kelvin);
}