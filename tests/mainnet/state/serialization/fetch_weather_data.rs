
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
    description: String,
}

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let weather_response: serde_json::Value = response.json().await?;
    
    let weather_data = WeatherData {
        temperature: weather_response["main"]["temp"].as_f64().unwrap_or(0.0),
        humidity: weather_response["main"]["humidity"].as_f64().unwrap_or(0.0),
        wind_speed: weather_response["wind"]["speed"].as_f64().unwrap_or(0.0),
        description: weather_response["weather"][0]["description"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
    };

    Ok(weather_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_data_success() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {"temp": 15.5, "humidity": 65.0},
                "wind": {"speed": 5.2},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let result = fetch_weather_data("test_key", "London").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.humidity, 65.0);
        assert_eq!(data.wind_speed, 5.2);
        assert_eq!(data.description, "clear sky");
    }

    #[tokio::test]
    async fn test_fetch_weather_data_failure() {
        let _m = mock("GET", "/data/2.5/weather?q=InvalidCity&appid=test_key&units=metric")
            .with_status(404)
            .create();

        let result = fetch_weather_data("test_key", "InvalidCity").await;
        assert!(result.is_err());
    }
}