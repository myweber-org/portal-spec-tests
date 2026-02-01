
use reqwest;
use serde_json::Value;
use std::error::Error;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: Value = response.json().await?;
    
    if weather_data.get("cod").and_then(|c| c.as_str()) != Some("200") {
        return Err(format!("Weather API error: {:?}", weather_data.get("message")).into());
    }
    
    Ok(weather_data)
}

pub fn extract_temperature(weather_data: &Value) -> Option<f64> {
    weather_data
        .get("main")?
        .get("temp")?
        .as_f64()
}

pub fn extract_weather_description(weather_data: &Value) -> Option<String> {
    let weather_array = weather_data.get("weather")?.as_array()?;
    weather_array
        .first()?
        .get("description")?
        .as_str()
        .map(|s| s.to_string())
}use reqwest;
use serde::Deserialize;
use std::error::Error;

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

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherResponse = response.json().await?;
    Ok(weather_data)
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
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let _weather = get_weather("test_key", "London").await.unwrap();
        mock.assert();
    }
}
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather_data = response.json::<WeatherData>().await?;
    
    Ok(weather_data)
}

#[tokio::main]
async fn main() {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", data.name);
            println!("Temperature: {:.1}°C", data.main.temp);
            println!("Humidity: {}%", data.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
}