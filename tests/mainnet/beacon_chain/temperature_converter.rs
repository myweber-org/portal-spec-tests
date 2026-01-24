
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{:.1}°C is equal to {:.1}°F", celsius_temp, fahrenheit_temp);
}
use std::io;

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature value:");

    let mut temperature = String::new();
    io::stdin()
        .read_line(&mut temperature)
        .expect("Failed to read line");

    let temperature: f64 = match temperature.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid number!");
            return;
        }
    };

    println!("Convert to: (C)elsius or (F)ahrenheit?");
    let mut unit = String::new();
    io::stdin()
        .read_line(&mut unit)
        .expect("Failed to read line");

    let unit = unit.trim().to_uppercase();

    match unit.as_str() {
        "C" => {
            let celsius = (temperature - 32.0) * 5.0 / 9.0;
            println!("{:.2}°F = {:.2}°C", temperature, celsius);
        }
        "F" => {
            let fahrenheit = temperature * 9.0 / 5.0 + 32.0;
            println!("{:.2}°C = {:.2}°F", temperature, fahrenheit);
        }
        _ => println!("Invalid unit. Please enter 'C' or 'F'."),
    }
}