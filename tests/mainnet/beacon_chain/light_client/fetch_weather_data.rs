use reqwest;
use serde_json::Value;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let json: Value = response.json().await?;
    
    match json["main"]["temp"].as_f64() {
        Some(temp) => Ok(temp),
        None => Err("Temperature data not found in response".into()),
    }
}
use reqwest;
use serde::Deserialize;
use std::error::Error;

const API_KEY: &str = "YOUR_API_KEY_HERE";
const BASE_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    name: String,
    main: MainData,
    weather: Vec<WeatherInfo>,
}

#[derive(Debug, Deserialize)]
pub struct MainData {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

#[derive(Debug, Deserialize)]
pub struct WeatherInfo {
    description: String,
}

pub async fn get_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("{}?q={}&appid={}&units=metric", BASE_URL, city, API_KEY);
    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather_data: WeatherData = response.json().await?;
        Ok(weather_data)
    } else {
        Err(format!("Failed to fetch weather data: {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_success() {
        let mock = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"feels_like":14.2,"humidity":65},"weather":[{"description":"clear sky"}]}"#)
            .create();

        let _guard = mockito::server_url();
        let result = get_weather("London").await;
        
        mock.assert();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
    }
}