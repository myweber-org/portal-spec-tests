use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather_data: WeatherData = response.json().await?;
    
    Ok(weather_data)
}

#[tokio::main]
async fn main() {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", data.name);
            println!("Temperature: {}°C", data.main.temp);
            println!("Humidity: {}%", data.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
}