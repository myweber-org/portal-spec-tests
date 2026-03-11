use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("Invalid API response: {0}")]
    ParseError(String),
    #[error("Location not found")]
    LocationNotFound,
    #[error("API key missing")]
    ApiKeyMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub description: String,
    pub timestamp: Instant,
}

pub struct WeatherCache {
    cache: Arc<RwLock<HashMap<String, (WeatherData, Instant)>>>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<WeatherData> {
        let cache = self.cache.read().unwrap();
        cache.get(location).and_then(|(data, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(&self, location: String, data: WeatherData) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(location, (data, Instant::now()));
    }

    pub fn clear_expired(&self) -> usize {
        let mut cache = self.cache.write().unwrap();
        let before_len = cache.len();
        cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.ttl);
        before_len - cache.len()
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl: u64) -> Self {
        Self {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: WeatherCache::new(cache_ttl),
        }
    }

    pub async fn fetch_weather(&self, location: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(location) {
            return Ok(cached);
        }

        let url = format!(
            "{}/current?location={}&apikey={}",
            self.base_url, location, self.api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if response.status() == 404 {
            return Err(WeatherError::LocationNotFound);
        }

        if !response.status().is_success() {
            return Err(WeatherError::NetworkError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let weather_data = WeatherData {
            temperature: json["main"]["temp"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid temperature".to_string()))?,
            humidity: json["main"]["humidity"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid humidity".to_string()))?,
            wind_speed: json["wind"]["speed"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid wind speed".to_string()))?,
            description: json["weather"][0]["description"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            timestamp: Instant::now(),
        };

        self.cache.insert(location.to_string(), weather_data.clone());
        Ok(weather_data)
    }

    pub fn cleanup_cache(&self) -> usize {
        self.cache.clear_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/current?location=London&apikey=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"main":{"temp":15.5,"humidity":65},"wind":{"speed":5.2},"weather":[{"description":"clear sky"}]}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        fetcher.base_url = server_url();

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65.0);
        assert_eq!(weather.wind_speed, 5.2);
        assert_eq!(weather.description, "clear sky");
    }

    #[test]
    fn test_cache_expiration() {
        let cache = WeatherCache::new(1);
        let test_data = WeatherData {
            temperature: 20.0,
            humidity: 50.0,
            wind_speed: 3.0,
            description: "test".to_string(),
            timestamp: Instant::now(),
        };

        cache.insert("TestCity".to_string(), test_data.clone());
        assert!(cache.get("TestCity").is_some());

        std::thread::sleep(Duration::from_secs(2));
        assert!(cache.get("TestCity").is_none());
    }
}