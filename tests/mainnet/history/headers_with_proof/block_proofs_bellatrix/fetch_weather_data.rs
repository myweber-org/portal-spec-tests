use std::collections::HashMap;
use std::time::{Duration, Instant};
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
    Network(String),
    #[error("Invalid API response: {0}")]
    Parse(String),
    #[error("City not found")]
    CityNotFound,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city)
            .filter(|(_, timestamp)| timestamp.elapsed() < self.ttl)
            .map(|(response, _)| response)
    }

    fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, Instant::now()));
    }
}

async fn fetch_weather(
    city: &str,
    api_key: &str,
    cache: &mut WeatherCache,
) -> Result<WeatherResponse, WeatherError> {
    if let Some(cached) = cache.get(city) {
        return Ok(cached.clone());
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| WeatherError::Network(e.to_string()))?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let weather_data: WeatherResponse = response
                .json()
                .await
                .map_err(|e| WeatherError::Parse(e.to_string()))?;
            
            cache.insert(city.to_string(), weather_data.clone());
            Ok(weather_data)
        }
        reqwest::StatusCode::NOT_FOUND => Err(WeatherError::CityNotFound),
        _ => Err(WeatherError::Network("Unexpected status code".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create_async()
            .await;

        let mut cache = WeatherCache::new(300);
        let api_key = "test_key";
        
        std::env::set_var("MOCK_URL", server.url());
        
        let result = fetch_weather("London", api_key, &mut cache).await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);

        mock.assert_async().await;
    }
}