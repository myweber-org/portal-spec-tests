use std::io::{self, Write};

struct WeatherData {
    city: String,
    temperature: f64,
    condition: String,
}

fn fetch_mock_weather(city: &str) -> Option<WeatherData> {
    let mock_data = [
        ("London", 15.5, "Cloudy"),
        ("Tokyo", 22.0, "Sunny"),
        ("New York", 18.0, "Rainy"),
        ("Paris", 20.5, "Clear"),
    ];

    for &(mock_city, temp, cond) in &mock_data {
        if mock_city.eq_ignore_ascii_case(city) {
            return Some(WeatherData {
                city: city.to_string(),
                temperature: temp,
                condition: cond.to_string(),
            });
        }
    }
    None
}

fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.city);
    println!("  Temperature: {:.1}°C", data.temperature);
    println!("  Condition: {}", data.condition);
}

fn main() {
    print!("Enter city name: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let city = input.trim();

    match fetch_mock_weather(city) {
        Some(weather) => display_weather(&weather),
        None => println!("Weather data not available for '{}'", city),
    }
}