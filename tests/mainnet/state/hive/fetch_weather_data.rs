
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
}use reqwest;
use serde::Deserialize;
use std::time::Duration;

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

#[derive(Debug)]
enum WeatherError {
    Network(reqwest::Error),
    Parse(String),
    RetryExhausted,
}

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, WeatherError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(WeatherError::Network)?;

    let mut attempts = 0;
    let max_attempts = 3;

    while attempts < max_attempts {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return response
                        .json::<WeatherResponse>()
                        .await
                        .map_err(|e| WeatherError::Parse(e.to_string()));
                } else if response.status().as_u16() == 429 {
                    attempts += 1;
                    if attempts < max_attempts {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
                        continue;
                    }
                }
                return Err(WeatherError::Parse(format!("HTTP error: {}", response.status())));
            }
            Err(e) => {
                attempts += 1;
                if attempts < max_attempts {
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
                    continue;
                }
                return Err(WeatherError::Network(e));
            }
        }
    }

    Err(WeatherError::RetryExhausted)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set");
    let city = "London";

    match fetch_weather_data(&api_key, city).await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                     weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => {
            eprintln!("Failed to fetch weather data: {:?}", e);
        }
    }

    Ok(())
}