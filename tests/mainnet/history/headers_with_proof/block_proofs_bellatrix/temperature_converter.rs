use std::io;

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
            let mut celsius = String::new();
            io::stdin()
                .read_line(&mut celsius)
                .expect("Failed to read line");
            let celsius: f64 = match celsius.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid temperature value.");
                    return;
                }
            };
            let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
            println!("{:.2}°C = {:.2}°F", celsius, fahrenheit);
        }
        2 => {
            println!("Enter temperature in Fahrenheit:");
            let mut fahrenheit = String::new();
            io::stdin()
                .read_line(&mut fahrenheit)
                .expect("Failed to read line");
            let fahrenheit: f64 = match fahrenheit.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid temperature value.");
                    return;
                }
            };
            let celsius = (fahrenheit - 32.0) * 5.0 / 9.0;
            println!("{:.2}°F = {:.2}°C", fahrenheit, celsius);
        }
        _ => println!("Invalid choice. Please select 1 or 2."),
    }
}