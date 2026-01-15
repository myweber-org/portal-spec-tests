
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    weather: Vec<Weather>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
    icon: String,
}

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather_data: WeatherData = response.json().await?;
    
    Ok(weather_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_fetch() {
        let api_key = "test_key";
        let city = "London";
        
        let result = get_weather(api_key, city).await;
        assert!(result.is_err());
    }
}
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherData {
    name: String,
    main: Main,
    weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

#[derive(Debug, Deserialize)]
struct Weather {
    main: String,
    description: String,
}

pub async fn get_current_weather(city: &str, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherData = response.json().await?;
    Ok(weather_data)
}

pub fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.name);
    println!("Temperature: {:.1}°C", data.main.temp);
    println!("Feels like: {:.1}°C", data.main.feels_like);
    println!("Humidity: {}%", data.main.humidity);
    
    if let Some(weather) = data.weather.first() {
        println!("Conditions: {} ({})", weather.main, weather.description);
    }
}