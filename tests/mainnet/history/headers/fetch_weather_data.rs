
use std::error::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    temperature: f64,
    humidity: u8,
    condition: String,
}

fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let mock_response = format!(
        r#"{{ "temperature": 22.5, "humidity": 65, "condition": "Sunny", "city": "{}" }}"#,
        city
    );
    let weather: WeatherData = serde_json::from_str(&mock_response)?;
    Ok(weather)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    let city = &args[1];
    let weather = fetch_weather(city)?;
    println!("Weather in {}:", city);
    println!("  Temperature: {}°C", weather.temperature);
    println!("  Humidity: {}%", weather.humidity);
    println!("  Condition: {}", weather.condition);
    Ok(())
}
use reqwest;
use serde_json::Value;
use std::error::Error;

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            base_url: String::from("https://api.openweathermap.org/data/2.5/weather"),
        }
    }

    pub async fn get_weather(&self, city: &str) -> Result<WeatherData, Box<dyn Error>> {
        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = reqwest::get(&url).await?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let json_data: Value = response.json().await?;
        
        let temperature = json_data["main"]["temp"]
            .as_f64()
            .ok_or("Failed to parse temperature")?;
        
        let humidity = json_data["main"]["humidity"]
            .as_i64()
            .ok_or("Failed to parse humidity")?;
        
        let description = json_data["weather"][0]["description"]
            .as_str()
            .ok_or("Failed to parse weather description")?
            .to_string();

        Ok(WeatherData {
            city: city.to_string(),
            temperature,
            humidity,
            description,
        })
    }
}

pub struct WeatherData {
    pub city: String,
    pub temperature: f64,
    pub humidity: i64,
    pub description: String,
}

impl WeatherData {
    pub fn display(&self) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {}",
            self.city, self.temperature, self.humidity, self.description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_weather_fetch_success() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {"temp": 22.5, "humidity": 65},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let fetcher = WeatherFetcher {
            api_key: "test_key".to_string(),
            base_url: server.url(),
        };

        let result = fetcher.get_weather("London").await;
        mock.assert();

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.city, "London");
        assert_eq!(data.temperature, 22.5);
        assert_eq!(data.humidity, 65);
        assert_eq!(data.description, "clear sky");
    }

    #[tokio::test]
    async fn test_weather_fetch_failure() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(404)
            .create();

        let fetcher = WeatherFetcher {
            api_key: "test_key".to_string(),
            base_url: server.url(),
        };

        let result = fetcher.get_weather("InvalidCity").await;
        mock.assert();

        assert!(result.is_err());
    }
}