use std::error::Error;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    city: String,
    temperature: f64,
    condition: String,
}

async fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("https://api.mockweather.example.com/current?city={}", city);
    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather: WeatherData = response.json().await?;
        Ok(weather)
    } else {
        Err(format!("Failed to fetch weather data for {}", city).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    
    let city = &args[1];
    match fetch_weather(city).await {
        Ok(data) => {
            println!("Weather in {}: {:.1}°C, {}", data.city, data.temperature, data.condition);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}