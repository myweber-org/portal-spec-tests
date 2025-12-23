use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("API response parsing failed: {0}")]
    ParseError(String),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub description: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: WeatherData,
    expires_at: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, CacheEntry>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();

        if let Some(entry) = self.cache.get(&cache_key) {
            if SystemTime::now() < entry.expires_at {
                return Ok(entry.data.clone());
            }
        }

        let weather_data = self.fetch_from_api(city).await?;
        
        let cache_entry = CacheEntry {
            data: weather_data.clone(),
            expires_at: SystemTime::now() + self.cache_duration,
        };
        
        self.cache.insert(cache_key, cache_entry);
        Ok(weather_data)
    }

    async fn fetch_from_api(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/weather?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if response.status() == 401 {
            return Err(WeatherError::InvalidApiKey);
        }

        if response.status() == 429 {
            return Err(WeatherError::RateLimitExceeded);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let main_data = json.get("main").ok_or_else(|| {
            WeatherError::ParseError("Missing 'main' field in response".to_string())
        })?;

        let wind_data = json.get("wind").ok_or_else(|| {
            WeatherError::ParseError("Missing 'wind' field in response".to_string())
        })?;

        let weather_array = json.get("weather").and_then(|w| w.as_array()).ok_or_else(|| {
            WeatherError::ParseError("Missing 'weather' array in response".to_string())
        })?;

        let description = weather_array
            .first()
            .and_then(|w| w.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(WeatherData {
            temperature: main_data["temp"].as_f64().unwrap_or(0.0),
            humidity: main_data["humidity"].as_f64().unwrap_or(0.0),
            pressure: main_data["pressure"].as_f64().unwrap_or(0.0),
            wind_speed: wind_data["speed"].as_f64().unwrap_or(0.0),
            description,
            timestamp: SystemTime::now(),
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_expired_entries(&mut self) {
        let now = SystemTime::now();
        self.cache.retain(|_, entry| entry.expires_at > now);
    }
}