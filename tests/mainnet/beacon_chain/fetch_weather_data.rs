use std::collections::HashMap;
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
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
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
        self.data.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(response)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, SystemTime::now()));
    }
}

async fn fetch_weather(
    client: &Client,
    cache: &mut WeatherCache,
    api_key: &str,
    city: &str,
) -> Result<WeatherResponse, WeatherError> {
    if let Some(cached) = cache.get(city) {
        return Ok(cached.clone());
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = client.get(&url).send().await?;
    
    match response.status() {
        reqwest::StatusCode::OK => {
            let weather_data: WeatherResponse = response.json().await?;
            cache.insert(city.to_string(), weather_data.clone());
            Ok(weather_data)
        }
        reqwest::StatusCode::UNAUTHORIZED => Err(WeatherError::InvalidApiKey),
        reqwest::StatusCode::NOT_FOUND => Err(WeatherError::CityNotFound),
        _ => Err(WeatherError::RequestFailed(
            response.error_for_status().unwrap_err(),
        )),
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

        let client = Client::new();
        let mut cache = WeatherCache::new(300);
        let result = fetch_weather(
            &client,
            &mut cache,
            "test_key",
            "London",
        ).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
        assert_eq!(data.main.humidity, 65);
    }
}