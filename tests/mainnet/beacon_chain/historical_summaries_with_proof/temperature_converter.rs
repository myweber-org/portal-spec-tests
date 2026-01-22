
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
            println!("Invalid input. Please enter a number.");
            return;
        }
    };

    let fahrenheit = celsius_to_fahrenheit(celsius);
    let kelvin = celsius_to_kelvin(celsius);

    println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
    println!("{:.2}°C = {:.2}K", celsius, kelvin);
}
use std::io;

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

struct Temperature {
    value: f64,
    unit: TemperatureUnit,
}

impl Temperature {
    fn new(value: f64, unit: TemperatureUnit) -> Self {
        Temperature { value, unit }
    }

    fn to_celsius(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => self.value,
            TemperatureUnit::Fahrenheit => (self.value - 32.0) * 5.0 / 9.0,
        }
    }

    fn to_fahrenheit(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => (self.value * 9.0 / 5.0) + 32.0,
            TemperatureUnit::Fahrenheit => self.value,
        }
    }

    fn convert_to(&self, target_unit: TemperatureUnit) -> Temperature {
        let converted_value = match target_unit {
            TemperatureUnit::Celsius => self.to_celsius(),
            TemperatureUnit::Fahrenheit => self.to_fahrenheit(),
        };
        Temperature::new(converted_value, target_unit)
    }

    fn display(&self) -> String {
        let unit_symbol = match self.unit {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        };
        format!("{:.2}{}", self.value, unit_symbol)
    }
}

fn parse_temperature_input(input: &str) -> Option<Temperature> {
    let input = input.trim().to_lowercase();
    
    if input.ends_with("c") {
        let value_str = &input[..input.len()-1];
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Temperature::new(value, TemperatureUnit::Celsius));
        }
    } else if input.ends_with("f") {
        let value_str = &input[..input.len()-1];
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Temperature::new(value, TemperatureUnit::Fahrenheit));
        }
    }
    
    None
}

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature with unit (e.g., '25c' or '77f'):");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    
    match parse_temperature_input(&input) {
        Some(temp) => {
            let converted = match temp.unit {
                TemperatureUnit::Celsius => temp.convert_to(TemperatureUnit::Fahrenheit),
                TemperatureUnit::Fahrenheit => temp.convert_to(TemperatureUnit::Celsius),
            };
            
            println!("Original: {}", temp.display());
            println!("Converted: {}", converted.display());
        }
        None => {
            println!("Invalid input format. Please use format like '25c' or '77f'");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        let temp = Temperature::new(0.0, TemperatureUnit::Celsius);
        assert_eq!(temp.to_fahrenheit(), 32.0);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        let temp = Temperature::new(32.0, TemperatureUnit::Fahrenheit);
        assert_eq!(temp.to_celsius(), 0.0);
    }

    #[test]
    fn test_convert_to() {
        let temp = Temperature::new(100.0, TemperatureUnit::Celsius);
        let converted = temp.convert_to(TemperatureUnit::Fahrenheit);
        assert_eq!(converted.value, 212.0);
    }
}