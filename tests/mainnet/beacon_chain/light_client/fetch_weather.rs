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
    weather: Vec<WeatherData>,
}

#[derive(Deserialize, Debug)]
struct MainData {
    temp: f64,
    humidity: u8,
}

#[derive(Deserialize, Debug)]
struct WeatherData {
    description: String,
}

pub struct WeatherCache {
    cache: HashMap<String, (SystemTime, WeatherData)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get_weather(
        &mut self,
        city: &str,
        api_key: &str,
    ) -> Result<String, WeatherError> {
        let cache_key = format!("{}-{}", city, api_key);

        if let Some((timestamp, data)) = self.cache.get(&cache_key) {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                return Ok(format!(
                    "{}: {} ({}°C)",
                    city, data.description, data.temp
                ));
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );

        let response: WeatherResponse = reqwest::get(&url)
            .await?
            .json()
            .await
            .map_err(|_| WeatherError::InvalidResponse)?;

        let weather_data = WeatherData {
            description: response.weather.first()
                .ok_or(WeatherError::InvalidResponse)?
                .description.clone(),
            temp: response.main.temp,
        };

        self.cache.insert(
            cache_key,
            (SystemTime::now(), weather_data.clone()),
        );

        Ok(format!(
            "{}: {} ({}°C)",
            city, weather_data.description, weather_data.temp
        ))
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_weather_fetch() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "weather": [{"description": "clear sky"}],
                "main": {"temp": 22.5, "humidity": 65}
            }"#)
            .create();

        let mut cache = WeatherCache::new(300);
        let result = cache.get_weather("London", "test_key").await;
        
        mock.assert();
        assert!(result.is_ok());
    }
}