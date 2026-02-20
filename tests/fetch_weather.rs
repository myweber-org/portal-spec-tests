use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Deserialize, Debug)]
struct MainData {
    temp: f64,
    humidity: u8,
}

struct WeatherCache {
    data: HashMap<String, (WeatherData, SystemTime)>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub city: String,
    pub temperature: f64,
    pub humidity: u8,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, city: &str) -> Option<WeatherData> {
        self.data.get(city).and_then(|(data, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, city: String, data: WeatherData) {
        self.data.insert(city, (data, SystemTime::now()));
    }
}

pub struct WeatherFetcher {
    api_key: String,
    cache: WeatherCache,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: WeatherCache::new(300),
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached);
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response: WeatherResponse = self
            .client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|_| WeatherError::InvalidResponse)?;

        let weather_data = WeatherData {
            city: response.name,
            temperature: response.main.temp,
            humidity: response.main.humidity,
        };

        self.cache.insert(city.to_string(), weather_data.clone());
        Ok(weather_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create_async()
            .await;

        let api_key = "test_key".to_string();
        let mut fetcher = WeatherFetcher::new(api_key);
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.city, "London");
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);

        mock.assert_async().await;
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = WeatherCache::new(60);
        let test_data = WeatherData {
            city: "TestCity".to_string(),
            temperature: 20.0,
            humidity: 50,
        };

        assert!(cache.get("TestCity").is_none());
        
        cache.insert("TestCity".to_string(), test_data.clone());
        let cached = cache.get("TestCity").unwrap();
        
        assert_eq!(cached.city, test_data.city);
        assert_eq!(cached.temperature, test_data.temperature);
        assert_eq!(cached.humidity, test_data.humidity);
    }
}