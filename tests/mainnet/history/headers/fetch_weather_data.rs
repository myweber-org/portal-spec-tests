
use std::error::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    temperature: f64,
    humidity: u8,
    condition: String,
}

fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let mock_response = format!(
        r#"{{ "temperature": 22.5, "humidity": 65, "condition": "Sunny", "city": "{}" }}"#,
        city
    );
    let weather: WeatherData = serde_json::from_str(&mock_response)?;
    Ok(weather)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    let city = &args[1];
    let weather = fetch_weather(city)?;
    println!("Weather in {}:", city);
    println!("  Temperature: {}°C", weather.temperature);
    println!("  Humidity: {}%", weather.humidity);
    println!("  Condition: {}", weather.condition);
    Ok(())
}