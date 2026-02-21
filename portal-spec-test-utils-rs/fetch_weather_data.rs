
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

pub async fn get_current_weather(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
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
    use mockito;

    #[tokio::test]
    async fn test_get_current_weather_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"feels_like":14.2,"humidity":65}}"#)
            .create();

        let api_key = "test_key";
        let city = "London";
        let url = server.url();

        let result = get_current_weather_with_url(api_key, city, &url).await;
        mock.assert();

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
    }

    async fn get_current_weather_with_url(api_key: &str, city: &str, base_url: &str) -> Result<WeatherData, Box<dyn Error>> {
        let url = format!(
            "{}/data/2.5/weather?q={}&appid={}&units=metric",
            base_url, city, api_key
        );

        let response = reqwest::get(&url).await?;
        let weather_data: WeatherData = response.json().await?;

        Ok(weather_data)
    }
}use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

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

#[derive(Error, Debug)]
enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned error status: {0}")]
    ApiError(reqwest::StatusCode),
    #[error("Failed after {0} retry attempts")]
    MaxRetriesExceeded(u32),
}

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, WeatherError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let client = reqwest::Client::new();
    
    for attempt in 0..=MAX_RETRIES {
        match client.get(&url).timeout(Duration::from_secs(10)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return response.json::<WeatherResponse>().await.map_err(WeatherError::from);
                } else if attempt < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                    continue;
                } else {
                    return Err(WeatherError::ApiError(response.status()));
                }
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                    continue;
                } else {
                    return Err(WeatherError::from(e));
                }
            }
        }
    }
    
    Err(WeatherError::MaxRetriesExceeded(MAX_RETRIES))
}

#[tokio::main]
async fn main() -> Result<(), WeatherError> {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let city = "London";
    
    match fetch_weather_data(&api_key, city).await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                     weather.name, weather.main.temp, weather.main.humidity);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to fetch weather data: {}", e);
            Err(e)
        }
    }
}