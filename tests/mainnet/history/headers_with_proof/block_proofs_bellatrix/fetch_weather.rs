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
}