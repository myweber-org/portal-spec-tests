
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
}use std::io;

fn main() {
    println!("Temperature Converter (Celsius to Fahrenheit)");
    println!("Enter temperature in Celsius:");

    let mut celsius = String::new();

    io::stdin()
        .read_line(&mut celsius)
        .expect("Failed to read line");

    let celsius: f64 = match celsius.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid number!");
            return;
        }
    };

    let fahrenheit = (celsius * 9.0 / 5.0) + 32.0;
    println!("{:.2}°C is equal to {:.2}°F", celsius, fahrenheit);
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