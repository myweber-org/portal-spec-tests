use std::collections::HashMap;
use std::time::{Duration, Instant};
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
    Network(#[from] reqwest::Error),
    #[error("API returned invalid data")]
    InvalidData,
    #[error("City not found")]
    CityNotFound,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city)
            .filter(|(_, timestamp)| timestamp.elapsed() < self.ttl)
            .map(|(data, _)| data)
    }

    fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, Instant::now()));
    }
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: WeatherCache::new(Duration::from_secs(300)),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached.clone());
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Err(WeatherError::CityNotFound);
        }

        let weather_data: WeatherResponse = response.json().await
            .map_err(|_| WeatherError::InvalidData)?;

        self.cache.insert(city.to_string(), weather_data.clone());
        
        Ok(weather_data)
    }

    pub fn display_weather(weather: &WeatherResponse) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity",
            weather.name,
            weather.main.temp,
            weather.main.humidity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_weather_fetch() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.api_key = "".to_string();
        
        let result = fetcher.get_weather("London").await;
        assert!(result.is_ok());
        
        if let Ok(weather) = result {
            assert_eq!(weather.name, "London");
            assert_eq!(weather.main.temp, 15.5);
            assert_eq!(weather.main.humidity, 65);
        }
    }
}