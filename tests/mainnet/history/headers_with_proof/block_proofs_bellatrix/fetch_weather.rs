use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use reqwest::Error as ReqwestError;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&mut self, city: &str, api_key: &str) -> Result<&WeatherResponse, CacheError> {
        let now = SystemTime::now();
        
        if let Some((cached_data, timestamp)) = self.data.get(city) {
            if now.duration_since(*timestamp).unwrap() < self.ttl {
                return Ok(cached_data);
            }
        }

        let weather = fetch_weather_data(city, api_key)?;
        self.data.insert(city.to_string(), (weather, now));
        
        Ok(&self.data.get(city).unwrap().0)
    }
}

fn fetch_weather_data(city: &str, api_key: &str) -> Result<WeatherResponse, CacheError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::blocking::get(&url)
        .map_err(|e| CacheError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(CacheError::ApiError(response.status().as_u16()));
    }
    
    response.json()
        .map_err(|e| CacheError::Parse(e.to_string()))
}

#[derive(Debug)]
enum CacheError {
    Network(String),
    Parse(String),
    ApiError(u16),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Network(msg) => write!(f, "Network error: {}", msg),
            CacheError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CacheError::ApiError(code) => write!(f, "API error with status code: {}", code),
        }
    }
}

impl std::error::Error for CacheError {}

fn display_weather(weather: &WeatherResponse) {
    println!("Weather in {}:", weather.name);
    println!("  Temperature: {:.1}°C", weather.main.temp);
    println!("  Humidity: {}%", weather.main.humidity);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("WEATHER_API_KEY")
        .unwrap_or_else(|_| "demo_key".to_string());
    
    let mut cache = WeatherCache::new(300);
    
    match cache.get("London", &api_key) {
        Ok(weather) => display_weather(weather),
        Err(e) => eprintln!("Failed to fetch weather: {}", e),
    }
    
    Ok(())
}use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

#[derive(Debug, Error)]
enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("City not found")]
    CityNotFound,
    #[error("Cache error: {0}")]
    CacheError(String),
}

struct WeatherCache {
    data: RwLock<HashMap<String, (WeatherResponse, SystemTime)>>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    fn get(&self, city: &str) -> Option<WeatherResponse> {
        let cache = self.data.read().unwrap();
        cache.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap() < self.ttl {
                Some(response.clone())
            } else {
                None
            }
        })
    }

    fn set(&self, city: String, response: WeatherResponse) -> Result<(), WeatherError> {
        let mut cache = self.data.write().map_err(|e| {
            WeatherError::CacheError(format!("Failed to acquire write lock: {}", e))
        })?;
        cache.insert(city, (response, SystemTime::now()));
        Ok(())
    }
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: Arc<WeatherCache>,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: Arc::new(WeatherCache::new(Duration::from_secs(300))),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached);
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        match response.status() {
            reqwest::StatusCode::OK => {
                let weather_data: WeatherResponse = response.json().await?;
                self.cache.set(city.to_string(), weather_data.clone())?;
                Ok(weather_data)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(WeatherError::InvalidApiKey),
            reqwest::StatusCode::NOT_FOUND => Err(WeatherError::CityNotFound),
            _ => Err(WeatherError::RequestFailed(
                response.error_for_status().unwrap_err()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = fetcher.fetch_weather("London").await;
        
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }

    #[tokio::test]
    async fn test_fetch_weather_city_not_found() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(404)
            .create();

        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = fetcher.fetch_weather("UnknownCity").await;
        
        assert!(matches!(result, Err(WeatherError::CityNotFound)));
    }
}