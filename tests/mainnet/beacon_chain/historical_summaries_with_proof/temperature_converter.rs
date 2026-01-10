
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