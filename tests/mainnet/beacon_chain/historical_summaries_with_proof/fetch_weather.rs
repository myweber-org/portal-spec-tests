use reqwest;
use serde::Deserialize;
use std::env;
use log::{info, error};

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherResponse, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather_data: WeatherResponse = response.json().await?;
    
    Ok(weather_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let api_key = env::var("WEATHER_API_KEY")
        .expect("WEATHER_API_KEY environment variable not set");
    
    let city = "London";
    
    match fetch_weather(&api_key, city).await {
        Ok(weather) => {
            info!("Weather data retrieved for {}", weather.name);
            println!("City: {}", weather.name);
            println!("Temperature: {:.1}°C", weather.main.temp);
            println!("Humidity: {}%", weather.main.humidity);
        }
        Err(e) => {
            error!("Failed to fetch weather data: {}", e);
            eprintln!("Error: {}", e);
        }
    }
    
    Ok(())
}