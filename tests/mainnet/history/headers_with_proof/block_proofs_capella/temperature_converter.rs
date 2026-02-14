
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{:.1}°C is equal to {:.1}°F", celsius_temp, fahrenheit_temp);
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
    Kelvin,
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
            TemperatureUnit::Kelvin => self.value - 273.15,
        }
    }

    fn to_fahrenheit(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => (self.value * 9.0 / 5.0) + 32.0,
            TemperatureUnit::Fahrenheit => self.value,
            TemperatureUnit::Kelvin => (self.value - 273.15) * 9.0 / 5.0 + 32.0,
        }
    }

    fn to_kelvin(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => self.value + 273.15,
            TemperatureUnit::Fahrenheit => (self.value - 32.0) * 5.0 / 9.0 + 273.15,
            TemperatureUnit::Kelvin => self.value,
        }
    }

    fn convert(&self, target_unit: TemperatureUnit) -> f64 {
        match target_unit {
            TemperatureUnit::Celsius => self.to_celsius(),
            TemperatureUnit::Fahrenheit => self.to_fahrenheit(),
            TemperatureUnit::Kelvin => self.to_kelvin(),
        }
    }
}

fn parse_unit(input: &str) -> Option<TemperatureUnit> {
    match input.trim().to_lowercase().as_str() {
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
        println!("\nEnter source temperature value and unit (e.g., '100 C'):");

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

        let source_unit = match parse_unit(parts[1]) {
            Some(unit) => unit,
            None => {
                println!("Invalid temperature unit. Use C, F, or K.");
                continue;
            }
        };

        let temperature = Temperature::new(value, source_unit);

        println!("Convert to which unit? (C/F/K):");
        let mut target_input = String::new();
        io::stdin().read_line(&mut target_input).expect("Failed to read line");

        let target_unit = match parse_unit(&target_input) {
            Some(unit) => unit,
            None => {
                println!("Invalid target unit.");
                continue;
            }
        };

        let converted_value = temperature.convert(target_unit);
        let target_unit_str = match target_unit {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
            TemperatureUnit::Kelvin => "K",
        };

        println!("Converted temperature: {:.2} {}", converted_value, target_unit_str);

        println!("\nConvert another temperature? (y/n):");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");

        if choice.trim().to_lowercase() != "y" {
            println!("Goodbye!");
            break;
        }
    }
}use std::io;

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature value:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let temperature: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid number");
            return;
        }
    };

    println!("Select conversion:");
    println!("1. Celsius to Fahrenheit");
    println!("2. Fahrenheit to Celsius");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    match choice.trim() {
        "1" => {
            let result = celsius_to_fahrenheit(temperature);
            println!("{:.2}°C = {:.2}°F", temperature, result);
        }
        "2" => {
            let result = fahrenheit_to_celsius(temperature);
            println!("{:.2}°F = {:.2}°C", temperature, result);
        }
        _ => println!("Invalid choice. Please enter 1 or 2."),
    }
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}