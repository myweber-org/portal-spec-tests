fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{}°C is equal to {}°F", celsius_temp, fahrenheit_temp);
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

    fn convert(&self, target_unit: TemperatureUnit) -> Temperature {
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
        let value_str = &input[..input.len() - 1];
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Temperature::new(value, TemperatureUnit::Celsius));
        }
    } else if input.ends_with("f") {
        let value_str = &input[..input.len() - 1];
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
                TemperatureUnit::Celsius => temp.convert(TemperatureUnit::Fahrenheit),
                TemperatureUnit::Fahrenheit => temp.convert(TemperatureUnit::Celsius),
            };
            
            println!("Original: {}", temp.display());
            println!("Converted: {}", converted.display());
        }
        None => {
            println!("Invalid input format. Please use format like '25c' or '77f'");
        }
    }
}