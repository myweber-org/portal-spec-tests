use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: u8,
    pub condition: String,
    pub wind_speed: f64,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, (WeatherData, SystemTime)>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some((data, timestamp)) = self.cache.get(city) {
            if timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration {
                return Ok(data.clone());
            }
        }

        let url = format!("{}/current?city={}&key={}", self.base_url, city, self.api_key);
        let response = reqwest::get(&url).await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherData = response.json().await?;
        self.cache.insert(city.to_string(), (weather_data.clone(), SystemTime::now()));
        
        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_weather_fetching() {
        let _m = mock("GET", "/current?city=London&key=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature": 15.5, "humidity": 65, "condition": "Cloudy", "wind_speed": 12.3}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url();
        
        let result = fetcher.get_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);
        assert_eq!(weather.condition, "Cloudy");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_cache_duration(Duration::from_secs(10));
        
        let weather_data = WeatherData {
            temperature: 20.0,
            humidity: 50,
            condition: "Sunny".to_string(),
            wind_speed: 5.0,
        };
        
        fetcher.cache.insert("Paris".to_string(), (weather_data.clone(), SystemTime::now()));
        
        let cached = fetcher.get_weather("Paris").await;
        assert!(cached.is_ok());
        assert_eq!(cached.unwrap().temperature, 20.0);
    }
}