
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

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
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
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_success() {
        let mock = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"feels_like":14.2,"humidity":65}}"#)
            .create();

        let api_key = "test_key";
        let city = "London";
        
        let result = get_weather(api_key, city).await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        
        mock.assert();
    }
}use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    #[error("Invalid API key")]
    InvalidApiKey,
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

    fn insert(&mut self, city: String, data: WeatherResponse) {
        self.data.insert(city, (data, Instant::now()));
    }
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: Arc<Mutex<WeatherCache>>,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: Arc::new(Mutex::new(WeatherCache::new(Duration::from_secs(300)))),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.lock().unwrap().get(city) {
            return Ok(cached.clone());
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        match response.status() {
            reqwest::StatusCode::OK => {
                let weather_data: WeatherResponse = response.json().await?;
                self.cache.lock().unwrap().insert(city.to_string(), weather_data.clone());
                Ok(weather_data)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(WeatherError::InvalidApiKey),
            reqwest::StatusCode::NOT_FOUND => Err(WeatherError::CityNotFound),
            _ => Err(WeatherError::Network(response.error_for_status().unwrap_err())),
        }
    }

    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "London".into()),
                Matcher::UrlEncoded("appid".into(), "test_key".into()),
                Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = fetcher.fetch_weather("London").await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
        assert_eq!(data.main.humidity, 65);
    }
}